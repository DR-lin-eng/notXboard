<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\Cache;
use App\Services\NodePlanAccessService;
use App\Services\SubscriptionQuotaService;
use App\Utils\CacheKey;
use App\Utils\Helper;

/**
 * App\Models\ServerNode
 *
 * @property int $id
 * @property int $user_id 服务器提供者 ID
 * @property string $name 节点名称
 * @property string $host 服务器地址
 * @property int $port 用户访问端口
 * @property int|null $service_port 服务端口(下发给节点，空则等于用户访问端口)
 * @property string $protocol 协议类型
 * @property string|null $location_code 节点国家/地区编码
 * @property string|null $location_name 节点国家/地区名称
 * @property array|null $settings 节点配置
 * @property int $traffic_limit 流量限制 (KB)
 * @property int $traffic_used 已使用流量 (KB)
 * @property float $traffic_multiplier 套餐扣费倍率
 * @property array|null $access_control 访问控制配置
 * @property array|null $free_quota_gb_by_trust_level Free quota per trust_level (GB/month), stored in access_control
 * @property string $status 节点状态
 * @property int|null $v2bx_node_id V2bX 节点 ID
 * @property array|null $v2bx_config V2bX 配置
 * @property string|null $v2bx_token V2bX 认证令牌
 * @property int $device_limit 设备数限制
 * @property int $connection_limit 连接数限制
 * @property int $speed_limit_up 上传限速 (Mbps)
 * @property int $speed_limit_down 下载限速 (Mbps)
 * @property int $cross_node_ip_limit 跨节点IP限制
 * @property int $concurrent_ip_limit 跨节点并发IP限制
 * @property bool $tcping_enabled TCPing 监控开关
 * @property string|null $tcping_host TCPing 主机
 * @property int|null $tcping_port TCPing 端口
 * @property int $tcping_interval_seconds TCPing 探测间隔
 * @property int $tcping_timeout_ms TCPing 超时时间
 * @property int $tcping_alert_after_seconds TCPing 告警阈值
 * @property int $tcping_recover_after_seconds TCPing 恢复阈值
 * @property string|null $tcping_last_status TCPing 最新状态
 * @property int|null $tcping_last_latency_ms TCPing 最新延迟
 * @property string|null $tcping_last_error TCPing 最新错误
 * @property int|null $tcping_last_sampled_at TCPing 最后采样时间
 * @property \Illuminate\Support\Carbon|null $created_at
 * @property \Illuminate\Support\Carbon|null $updated_at
 *
 * @property-read User $owner 节点所有者
 * @property-read \Illuminate\Database\Eloquent\Collection<int, AuditRule> $auditRules 审计规则
 * @property-read \Illuminate\Database\Eloquent\Collection<int, UserOnlineSession> $onlineSessions 在线会话
 * @property-read \Illuminate\Database\Eloquent\Collection<int, AuditLog> $auditLogs 审计日志
 * @property-read \Illuminate\Database\Eloquent\Collection<int, NodeTrafficRecord> $trafficRecords 流量记录
 * @property-read \Illuminate\Database\Eloquent\Collection<int, User> $authorizedUsers 授权用户
 */
class ServerNode extends Model
{
    use HasFactory;

    protected $fillable = [
        'user_id',
        'name',
        'host',
        'port',
        'service_port',
        'protocol',
        'location_code',
        'location_name',
        'settings',
        'traffic_limit',
        'traffic_used',
        'traffic_multiplier',
        'access_control',
        'status',
        'v2bx_node_id',
        'v2bx_config',
        'v2bx_token',
        'device_limit',
        'connection_limit',
        'speed_limit_up',
        'speed_limit_down',
        'cross_node_ip_limit',
        'concurrent_ip_limit',
        'tcping_enabled',
        'tcping_host',
        'tcping_port',
        'tcping_interval_seconds',
        'tcping_timeout_ms',
        'tcping_alert_after_seconds',
        'tcping_recover_after_seconds',
        'tcping_last_status',
        'tcping_last_latency_ms',
        'tcping_last_error',
        'tcping_last_sampled_at',
        'tcping_outage_since',
        'tcping_recovered_since',
    ];

    protected $casts = [
        'settings' => 'array',
        'access_control' => 'array',
        'v2bx_config' => 'array',
        'traffic_limit' => 'integer',
        'traffic_used' => 'integer',
        'traffic_multiplier' => 'float',
        'device_limit' => 'integer',
        'connection_limit' => 'integer',
        'speed_limit_up' => 'integer',
        'speed_limit_down' => 'integer',
        'cross_node_ip_limit' => 'integer',
        'concurrent_ip_limit' => 'integer',
        'service_port' => 'integer',
        'tcping_enabled' => 'boolean',
        'tcping_port' => 'integer',
        'tcping_interval_seconds' => 'integer',
        'tcping_timeout_ms' => 'integer',
        'tcping_alert_after_seconds' => 'integer',
        'tcping_recover_after_seconds' => 'integer',
        'tcping_last_latency_ms' => 'integer',
        'tcping_last_sampled_at' => 'integer',
        'tcping_outage_since' => 'integer',
        'tcping_recovered_since' => 'integer',
    ];

    // 协议类型常量
    public const PROTOCOL_VMESS = 'vmess';
    public const PROTOCOL_VLESS = 'vless';
    public const PROTOCOL_TROJAN = 'trojan';
    public const PROTOCOL_SHADOWSOCKS = 'shadowsocks';
    public const PROTOCOL_HYSTERIA = 'hysteria';
    public const PROTOCOL_HYSTERIA2 = 'hysteria2';
    public const PROTOCOL_TUIC = 'tuic';
    public const PROTOCOL_ANYTLS = 'anytls';
    public const PROTOCOL_SOCKS = 'socks';
    public const PROTOCOL_HTTP = 'http';
    public const PROTOCOL_NAIVE = 'naive';
    public const PROTOCOL_MIERU = 'mieru';

    public const SUPPORTED_PROTOCOLS = [
        self::PROTOCOL_VMESS,
        self::PROTOCOL_VLESS,
        self::PROTOCOL_TROJAN,
        self::PROTOCOL_SHADOWSOCKS,
        self::PROTOCOL_HYSTERIA,
        self::PROTOCOL_HYSTERIA2,
        self::PROTOCOL_TUIC,
        self::PROTOCOL_ANYTLS,
        self::PROTOCOL_SOCKS,
        self::PROTOCOL_HTTP,
        self::PROTOCOL_NAIVE,
        self::PROTOCOL_MIERU,
    ];

    /**
     * V2bX 面板客户端当前支持的一键对接协议（以 V2bX 实际实现为准）
     */
    public const V2BX_SUPPORTED_PROTOCOLS = [
        self::PROTOCOL_VMESS,
        self::PROTOCOL_VLESS,
        self::PROTOCOL_TROJAN,
        self::PROTOCOL_SHADOWSOCKS,
        self::PROTOCOL_HYSTERIA,
        self::PROTOCOL_HYSTERIA2,
        self::PROTOCOL_TUIC,
        self::PROTOCOL_ANYTLS,
    ];

    /**
     * 这些协议在 V2bX 中为强制 TLS，证书模式为 none 时会启动失败
     */
    public const V2BX_TLS_REQUIRED_PROTOCOLS = [
        self::PROTOCOL_TROJAN,
        self::PROTOCOL_HYSTERIA,
        self::PROTOCOL_HYSTERIA2,
        self::PROTOCOL_TUIC,
        self::PROTOCOL_ANYTLS,
    ];

    /**
     * 这些协议推荐自动附带证书参数，减少首次对接失败
     */
    public const V2BX_AUTO_CERT_PROTOCOLS = [
        self::PROTOCOL_VMESS,
        self::PROTOCOL_VLESS,
        self::PROTOCOL_TROJAN,
        self::PROTOCOL_HYSTERIA,
        self::PROTOCOL_HYSTERIA2,
        self::PROTOCOL_TUIC,
        self::PROTOCOL_ANYTLS,
    ];

    // 状态常量
    public const STATUS_ACTIVE = 'active';
    public const STATUS_INACTIVE = 'inactive';
    public const STATUS_MAINTENANCE = 'maintenance';
    public const STATUS_DEPLOYING = 'deploying';

    // V2bX 回传超时时间（秒）
    public const ONLINE_TIMEOUT_SECONDS = 300;

    /**
     * 关联节点所有者
     */
    public function owner(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    /**
     * 关联审计规则
     */
    public function auditRules(): HasMany
    {
        return $this->hasMany(AuditRule::class, 'node_id', 'id');
    }

    /**
     * 关联在线会话
     */
    public function onlineSessions(): HasMany
    {
        return $this->hasMany(UserOnlineSession::class, 'node_id', 'id');
    }

    /**
     * 关联审计日志
     */
    public function auditLogs(): HasMany
    {
        return $this->hasMany(AuditLog::class, 'node_id', 'id');
    }

    /**
     * 关联流量记录
     */
    public function trafficRecords(): HasMany
    {
        return $this->hasMany(NodeTrafficRecord::class, 'node_id', 'id');
    }

    public function trafficUsageLogs(): HasMany
    {
        return $this->hasMany(UserTrafficUsageLog::class, 'node_id', 'id');
    }

    public function tcpingSamples(): HasMany
    {
        return $this->hasMany(TcpingSample::class, 'node_id', 'id');
    }

    public function isUdpBasedProtocol(): bool
    {
        return in_array(strtolower((string) $this->protocol), [
            self::PROTOCOL_HYSTERIA,
            self::PROTOCOL_HYSTERIA2,
            self::PROTOCOL_TUIC,
        ], true);
    }

    /**
     * TCPing 只能探测 TCP 端口。对 UDP 协议节点（hysteria/hysteria2/tuic），
     * 若未配置独立的 TCPing 探测端口，则默认不下发探测目标。
     */
    public function isTcpingMonitorable(): bool
    {
        if (!$this->isUdpBasedProtocol()) {
            return true;
        }

        if ($this->tcping_port === null) {
            return false;
        }

        $probePort = (int) $this->tcping_port;
        if ($probePort <= 0) {
            return false;
        }

        return true;
    }

    public function tcpingAlerts(): HasMany
    {
        return $this->hasMany(TcpingAlert::class, 'node_id', 'id');
    }

    /**
     * 关联授权用户
     */
    public function authorizedUsers(): BelongsToMany
    {
        return $this->belongsToMany(User::class, 'user_node_access', 'node_id', 'user_id')
                    ->withPivot('access_type', 'granted_at');
    }

    /**
     * 检查节点是否活跃
     */
    public function isActive(): bool
    {
        return $this->status === self::STATUS_ACTIVE;
    }

    public function getLastReportAt(): ?int
    {
        $cacheKey = CacheKey::get('SERVER_SERVER_NODE_LAST_LOAD_AT', $this->id);
        $lastReportAt = Cache::get($cacheKey);
        if ($lastReportAt === null) {
            return null;
        }
        if (!is_numeric($lastReportAt)) {
            return null;
        }

        return (int) $lastReportAt;
    }

    public function getOnlineTimeoutSeconds(): int
    {
        $pushInterval = (int) admin_setting('server_push_interval', 60);
        return max(self::ONLINE_TIMEOUT_SECONDS, $pushInterval * 3);
    }

    public function isReportedOnline(): bool
    {
        $lastReportAt = $this->getLastReportAt();
        if ($lastReportAt === null) {
            return false;
        }

        return (time() - $lastReportAt) <= $this->getOnlineTimeoutSeconds();
    }

    public function getOnlineStatus(): string
    {
        return $this->isReportedOnline() ? 'online' : 'offline';
    }

    public static function supportsV2bxDeploy(?string $protocol): bool
    {
        if (!$protocol) {
            return false;
        }
        return in_array(strtolower($protocol), self::V2BX_SUPPORTED_PROTOCOLS, true);
    }

    public static function requiresTlsForV2bx(?string $protocol): bool
    {
        if (!$protocol) {
            return false;
        }
        return in_array(strtolower($protocol), self::V2BX_TLS_REQUIRED_PROTOCOLS, true);
    }

    public static function prefersAutoCertForV2bx(?string $protocol): bool
    {
        if (!$protocol) {
            return false;
        }
        return in_array(strtolower($protocol), self::V2BX_AUTO_CERT_PROTOCOLS, true);
    }

    /**
     * 检查流量是否超限
     */
    public function isTrafficExceeded(): bool
    {
        if ($this->traffic_limit <= 0) {
            return false;
        }
        
        return $this->traffic_used >= $this->traffic_limit;
    }

    /**
     * 获取流量使用百分比
     */
    public function getTrafficUsagePercentage(): float
    {
        if ($this->traffic_limit <= 0) {
            return 0;
        }
        
        return min(100, ($this->traffic_used / $this->traffic_limit) * 100);
    }

    /**
     * 获取剩余流量
     */
    public function getRemainingTraffic(): int
    {
        if ($this->traffic_limit <= 0) {
            return PHP_INT_MAX;
        }
        
        return max(0, $this->traffic_limit - $this->traffic_used);
    }

    /**
     * 检查用户是否有访问权限
     */
    public function canUserAccess(User $user): bool
    {
        // Super admin always has access to all nodes
        if ((bool) $user->is_super_admin) {
            return true;
        }

        // Node owner always has access
        if ($this->user_id === $user->id) {
            return true;
        }

        // Node-level blacklist takes precedence
        $isBlacklisted = DB::table('user_node_blacklist')
            ->where('node_id', $this->id)
            ->where('user_id', $user->id)
            ->exists();
        if ($isBlacklisted) {
            return false;
        }

        if ((bool) $user->banned) {
            return false;
        }

        $expiredAt = $user->expired_at;
        if ($expiredAt !== null && (int) $expiredAt < time()) {
            return false;
        }

        $hasIndividualAccess = $this->authorizedUsers()
            ->where('user_id', $user->id)
            ->exists();

        $accessControl = $this->access_control ?? [];
        $minTrustLevel = $accessControl['min_trust_level'] ?? null;
        $hasGroupAccess = $minTrustLevel !== null
            && (int) $user->trust_level >= (int) $minTrustLevel;

        $hasPlanAccess = app(NodePlanAccessService::class)
            ->userHasActiveAccessToNode($user, (int) $this->id);
        $hasRemainingQuota = $hasPlanAccess && app(SubscriptionQuotaService::class)
            ->userHasRemainingQuotaForNode($user, (int) $this->id);

        return $hasIndividualAccess || $hasGroupAccess || $hasRemainingQuota;
    }

    protected function canUserAccessByPlanFreeQuota(User $user): bool
    {
        $trustLevel = (int) ($user->trust_level ?? 0);
        $levelKey = (string) $trustLevel;

        $plans = $this->getPlanFreeQuotaPlans();

        foreach ($plans as $p) {
            $minTl = $p->min_trust_level !== null ? (int) $p->min_trust_level : null;
            if ($minTl !== null && $trustLevel < $minTl) {
                continue;
            }

            $map = is_array($p->free_quota_gb_by_trust_level) ? $p->free_quota_gb_by_trust_level : json_decode((string) $p->free_quota_gb_by_trust_level, true);
            if (!is_array($map)) {
                continue;
            }
            $quotaGb = $map[$levelKey] ?? ($map[$trustLevel] ?? null);
            if ($quotaGb === null) {
                continue;
            }
            $quotaGb = (float) $quotaGb;
            if ($quotaGb <= 0) {
                continue;
            }
            $quotaKb = (int) floor($quotaGb * 1024 * 1024);
            if ($quotaKb <= 0) {
                continue;
            }

            $ym = now()->format('Ym');
            $usedKb = Cache::remember("plan:{$p->id}:user:{$user->id}:used:{$ym}", 60, function () use ($p, $user) {
                $nodeIds = json_decode((string) $p->node_ids, true);
                if (!is_array($nodeIds) || empty($nodeIds)) {
                    return 0;
                }
                $nodeIds = collect($nodeIds)->map(fn ($v) => (int) $v)->filter(fn ($v) => $v > 0)->unique()->values()->all();
                if (empty($nodeIds)) {
                    return 0;
                }
                $start = now()->startOfMonth()->toDateString();
                $end = now()->endOfMonth()->toDateString();
                return (int) DB::table('node_traffic_records')
                    ->where('user_id', $user->id)
                    ->whereIn('node_id', $nodeIds)
                    ->whereBetween('record_date', [$start, $end])
                    ->sum(DB::raw('upload_traffic + download_traffic'));
            });

            if ($usedKb < $quotaKb) {
                return true;
            }
        }

        return false;
    }

    public function getPlanFreeQuotaTrustLevels(): array
    {
        $levels = [];
        foreach ($this->getPlanFreeQuotaPlans() as $p) {
            $map = is_array($p->free_quota_gb_by_trust_level) ? $p->free_quota_gb_by_trust_level : json_decode((string) $p->free_quota_gb_by_trust_level, true);
            if (!is_array($map)) {
                continue;
            }
            foreach ($map as $k => $v) {
                $level = (int) $k;
                $gb = (float) $v;
                if ($level >= 0 && $level <= 4 && $gb > 0) {
                    $levels[] = $level;
                }
            }
        }
        $levels = array_values(array_unique($levels));
        sort($levels);
        return $levels;
    }

    protected function getPlanFreeQuotaPlans()
    {
        return Cache::remember("node:{$this->id}:free_quota_plans", 600, function () {
            return DB::table('v2_plan')
                ->select(['id', 'min_trust_level', 'node_ids', 'free_quota_gb_by_trust_level'])
                ->where('scope', \App\Models\Plan::SCOPE_NODE)
                ->where('show', 1)
                ->where('sell', 1)
                ->whereNotNull('free_quota_gb_by_trust_level')
                ->whereRaw('JSON_CONTAINS(node_ids, CAST(? AS JSON))', [(string) $this->id])
                ->orderByDesc('id')
                ->get();
        });
    }

    public function getUserMonthlyTrafficUsedKb(int $userId, ?Carbon $now = null): int
    {
        $now = $now ?? now();
        $start = $now->copy()->startOfMonth()->toDateString();
        $end = $now->copy()->endOfMonth()->toDateString();

        return (int) DB::table('node_traffic_records')
            ->where('node_id', $this->id)
            ->where('user_id', $userId)
            ->whereBetween('record_date', [$start, $end])
            ->sum(DB::raw('upload_traffic + download_traffic'));
    }

    /**
     * 更新流量使用量
     */
    public function updateTrafficUsage(int $upload, int $download): void
    {
        $this->increment('traffic_used', $upload + $download);
    }

    /**
     * 获取在线用户数
     */
    public function getOnlineUserCount(): int
    {
        return $this->onlineSessions()
                    ->where('last_activity', '>=', now()->subMinutes(5))
                    ->distinct('user_id')
                    ->count();
    }

    /**
     * 获取节点配置用于 V2bX
     */
    public function getV2bXConfig(): array
    {
        $config = array_merge([
            'id' => $this->v2bx_node_id ?? $this->id,
            'name' => $this->name,
            'host' => $this->host,
            'port' => $this->getEffectiveServicePort(),
            'service_port' => $this->getEffectiveServicePort(),
            'access_port' => $this->port,
            'protocol' => $this->protocol,
            'settings' => $this->settings ?? [],
        ], $this->v2bx_config ?? []);

        // Always use service_port as node-side listen port.
        $config['port'] = $this->getEffectiveServicePort();
        $config['service_port'] = $this->getEffectiveServicePort();
        $config['access_port'] = (int) $this->port;

        return $config;
    }

    /**
     * Build panel response payload for V2bX /api/v1/server/UniProxy/config.
     * Keep field names aligned with V2bX parser structs in api/panel/node.go.
     */
    public function getV2bXPanelConfig(): array
    {
        $protocol = strtolower((string) $this->protocol);
        $normalizedType = Server::normalizeType($protocol) ?? Server::TYPE_VMESS;
        $settings = $this->getNormalizedProtocolSettings($normalizedType, $protocol);

        $base = [
            'host' => (string) $this->host,
            'server_port' => (int) $this->getEffectiveServicePort(),
            'server_name' => $this->resolvePanelServerName($settings),
        ];

        return match ($protocol) {
            self::PROTOCOL_VMESS => [
                ...$base,
                'tls' => (int) ($settings['tls'] ?? 0),
                'network' => (string) ($settings['network'] ?? 'tcp'),
                'network_settings' => self::normalizeObjectValue(
                    $settings['network_settings'] ?? ($settings['networkSettings'] ?? null),
                    true
                ),
                'tls_settings' => self::normalizeObjectValue(
                    $settings['tls_settings'] ?? ($settings['tlsSettings'] ?? null),
                    true
                ),
            ],
            self::PROTOCOL_VLESS => [
                ...$base,
                'tls' => (int) ($settings['tls'] ?? 0),
                'network' => (string) ($settings['network'] ?? 'tcp'),
                'network_settings' => self::normalizeObjectValue(
                    $settings['network_settings'] ?? ($settings['networkSettings'] ?? null),
                    true
                ),
                'flow' => $settings['flow'] ?? null,
                'tls_settings' => self::normalizeObjectValue(
                    (int) ($settings['tls'] ?? 0) === 2
                        ? ($settings['reality_settings'] ?? null)
                        : ($settings['tls_settings'] ?? ($settings['tlsSettings'] ?? null)),
                    true
                ),
            ],
            self::PROTOCOL_TROJAN => [
                ...$base,
                'network' => (string) ($settings['network'] ?? 'tcp'),
                'networkSettings' => self::normalizeObjectValue(
                    $settings['network_settings'] ?? ($settings['networkSettings'] ?? null),
                    true
                ),
            ],
            self::PROTOCOL_SHADOWSOCKS => $this->buildV2bXShadowsocksConfig($base, $settings),
            self::PROTOCOL_HYSTERIA,
            self::PROTOCOL_HYSTERIA2 => $this->buildV2bXHysteriaConfig($base, $settings, $protocol),
            self::PROTOCOL_TUIC => [
                ...$base,
                'congestion_control' => (string) ($settings['congestion_control'] ?? 'cubic'),
                'zero_rtt_handshake' => (bool) ($settings['zero_rtt_handshake'] ?? false),
                'heartbeat' => (string) ($settings['heartbeat'] ?? '10s'),
            ],
            self::PROTOCOL_ANYTLS => [
                ...$base,
                'padding_scheme' => is_array($settings['padding_scheme'] ?? null)
                    ? array_values($settings['padding_scheme'])
                    : [],
            ],
            default => $base,
        };
    }

    private function getNormalizedProtocolSettings(string $normalizedType, string $rawProtocol): array
    {
        $defaults = Server::getProtocolSettingTemplateForType($normalizedType);
        $defaults = is_array($defaults) ? $defaults : [];
        $rawSettings = is_array($this->settings) ? $this->settings : [];
        $settings = array_replace_recursive($defaults, $rawSettings);

        if ($rawProtocol === self::PROTOCOL_HYSTERIA2) {
            $settings['version'] = 2;
        }

        return is_array($settings) ? $settings : [];
    }

    private function resolvePanelServerName(array $settings): string
    {
        $candidates = [
            data_get($settings, 'server_name'),
            data_get($settings, 'tls.server_name'),
            data_get($settings, 'tls_settings.server_name'),
            data_get($settings, 'reality_settings.server_name'),
        ];

        foreach ($candidates as $candidate) {
            if (!is_string($candidate)) {
                continue;
            }
            $value = trim($candidate);
            if ($value !== '') {
                return $value;
            }
        }

        return (string) $this->host;
    }

    private static function normalizeObjectValue(mixed $value, bool $defaultEmptyObject = false): mixed
    {
        if ($value instanceof \stdClass) {
            return $value;
        }

        if (is_string($value)) {
            $decoded = json_decode($value, true);
            if (json_last_error() === JSON_ERROR_NONE) {
                $value = $decoded;
            } else {
                return $defaultEmptyObject ? (object) [] : null;
            }
        }

        if (is_array($value)) {
            if ($value === []) {
                return $defaultEmptyObject ? (object) [] : null;
            }
            if (array_is_list($value)) {
                return $defaultEmptyObject ? (object) [] : $value;
            }
            return $value;
        }

        return $defaultEmptyObject ? (object) [] : null;
    }

    private function buildV2bXShadowsocksConfig(array $base, array $settings): array
    {
        $cipher = (string) ($settings['cipher'] ?? 'aes-128-gcm');
        return [
            ...$base,
            'cipher' => $cipher,
            'server_key' => match ($cipher) {
                '2022-blake3-aes-128-gcm' => Helper::getServerKey($this->created_at, 16),
                '2022-blake3-aes-256-gcm' => Helper::getServerKey($this->created_at, 32),
                default => null,
            },
        ];
    }

    private function buildV2bXHysteriaConfig(array $base, array $settings, string $protocol): array
    {
        $version = (int) ($settings['version'] ?? ($protocol === self::PROTOCOL_HYSTERIA2 ? 2 : 1));
        if ($protocol === self::PROTOCOL_HYSTERIA2) {
            $version = 2;
        }

        $config = [
            ...$base,
            'version' => $version,
            'up_mbps' => (int) data_get($settings, 'bandwidth.up', 0),
            'down_mbps' => (int) data_get($settings, 'bandwidth.down', 0),
        ];

        if ($version >= 2) {
            $obfsEnabled = (bool) data_get($settings, 'obfs.open', false);
            return [
                ...$config,
                'obfs' => $obfsEnabled ? (string) data_get($settings, 'obfs.type', 'salamander') : null,
                'obfs-password' => $obfsEnabled ? (string) data_get($settings, 'obfs.password', '') : null,
                'ignore_client_bandwidth' => (bool) ($settings['ignore_client_bandwidth'] ?? false),
            ];
        }

        $obfs = (string) data_get($settings, 'obfs.password', '');
        return [
            ...$config,
            'obfs' => $obfs !== '' ? $obfs : null,
        ];
    }

    public function getEffectiveServicePort(): int
    {
        $servicePort = (int) ($this->service_port ?? 0);
        if ($servicePort > 0) {
            return $servicePort;
        }

        return (int) $this->port;
    }

    public function getEffectiveTrafficMultiplier(): float
    {
        $multiplier = (float) ($this->traffic_multiplier ?? 1);
        return $multiplier > 0 ? $multiplier : 1.0;
    }

    public function getTcpingTargetHost(): string
    {
        return (string) ($this->tcping_host ?: $this->host);
    }

    public function getTcpingTargetPort(): int
    {
        return (int) (($this->tcping_port ?: 0) > 0 ? $this->tcping_port : $this->port);
    }
}
