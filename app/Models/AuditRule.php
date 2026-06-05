<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * App\Models\AuditRule
 *
 * @property int $id
 * @property int $node_id 关联节点 ID
 * @property string $rule_type 规则类型
 * @property string $rule_pattern 规则模式
 * @property string $action 动作
 * @property bool $is_active 是否启用
 * @property \Illuminate\Support\Carbon|null $created_at
 * @property \Illuminate\Support\Carbon|null $updated_at
 *
 * @property-read ServerNode $node 关联节点
 * @property-read \Illuminate\Database\Eloquent\Collection<int, AuditLog> $auditLogs 审计日志
 */
class AuditRule extends Model
{
    use HasFactory;

    protected $fillable = [
        'node_id',
        'rule_type',
        'rule_pattern',
        'action',
        'is_active',
    ];

    protected $casts = [
        'is_active' => 'boolean',
    ];

    // 规则类型常量
    public const TYPE_DOMAIN = 'domain';
    public const TYPE_PROTOCOL = 'protocol';
    public const TYPE_IP = 'ip';

    // 动作常量
    public const ACTION_BLOCK = 'block';
    public const ACTION_ALLOW = 'allow';
    public const ACTION_LOG = 'log';

    /**
     * 关联节点
     */
    public function node(): BelongsTo
    {
        return $this->belongsTo(ServerNode::class, 'node_id', 'id');
    }

    /**
     * 关联审计日志
     */
    public function auditLogs(): HasMany
    {
        return $this->hasMany(AuditLog::class, 'rule_id', 'id');
    }

    /**
     * 检查规则是否匹配
     */
    public function matches(string $target): bool
    {
        if (!$this->is_active) {
            return false;
        }

        switch ($this->rule_type) {
            case self::TYPE_DOMAIN:
                return $this->matchesDomain($target);
            case self::TYPE_PROTOCOL:
                return $this->matchesProtocol($target);
            case self::TYPE_IP:
                return $this->matchesIP($target);
            default:
                return false;
        }
    }

    /**
     * 匹配域名规则
     */
    private function matchesDomain(string $domain): bool
    {
        return preg_match($this->buildWildcardRegex($this->rule_pattern, true), $domain) === 1;
    }

    /**
     * 匹配协议规则
     */
    private function matchesProtocol(string $protocol): bool
    {
        return strcasecmp($this->rule_pattern, $protocol) === 0;
    }

    /**
     * 匹配IP规则
     */
    private function matchesIP(string $ip): bool
    {
        // 支持CIDR格式
        if (strpos($this->rule_pattern, '/') !== false) {
            return $this->ipInCIDR($ip, $this->rule_pattern);
        }
        
        return preg_match($this->buildWildcardRegex($this->rule_pattern), $ip) === 1;
    }

    private function buildWildcardRegex(string $pattern, bool $caseInsensitive = false): string
    {
        $quoted = preg_quote($pattern, '/');
        $quoted = str_replace('\*', '.*', $quoted);

        return '/^' . $quoted . '$/' . ($caseInsensitive ? 'i' : '');
    }

    /**
     * 检查IP是否在CIDR范围内
     */
    private function ipInCIDR(string $ip, string $cidr): bool
    {
        list($subnet, $mask) = explode('/', $cidr);
        
        if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4)) {
            return $this->ipv4InCIDR($ip, $subnet, (int)$mask);
        } elseif (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV6)) {
            return $this->ipv6InCIDR($ip, $subnet, (int)$mask);
        }
        
        return false;
    }

    /**
     * 检查IPv4是否在CIDR范围内
     */
    private function ipv4InCIDR(string $ip, string $subnet, int $mask): bool
    {
        $ipLong = ip2long($ip);
        $subnetLong = ip2long($subnet);
        $maskLong = -1 << (32 - $mask);
        
        return ($ipLong & $maskLong) === ($subnetLong & $maskLong);
    }

    /**
     * 检查IPv6是否在CIDR范围内
     */
    private function ipv6InCIDR(string $ip, string $subnet, int $mask): bool
    {
        $ipBin = inet_pton($ip);
        $subnetBin = inet_pton($subnet);
        
        if ($ipBin === false || $subnetBin === false) {
            return false;
        }
        
        $bytesToCheck = intval($mask / 8);
        $bitsToCheck = $mask % 8;
        
        // 检查完整字节
        for ($i = 0; $i < $bytesToCheck; $i++) {
            if ($ipBin[$i] !== $subnetBin[$i]) {
                return false;
            }
        }
        
        // 检查剩余位
        if ($bitsToCheck > 0 && $bytesToCheck < 16) {
            $maskByte = 0xFF << (8 - $bitsToCheck);
            if ((ord($ipBin[$bytesToCheck]) & $maskByte) !== (ord($subnetBin[$bytesToCheck]) & $maskByte)) {
                return false;
            }
        }
        
        return true;
    }

    /**
     * 获取规则描述
     */
    public function getDescription(): string
    {
        $typeNames = [
            self::TYPE_DOMAIN => '域名',
            self::TYPE_PROTOCOL => '协议',
            self::TYPE_IP => 'IP地址',
        ];
        
        $actionNames = [
            self::ACTION_BLOCK => '阻止',
            self::ACTION_ALLOW => '允许',
            self::ACTION_LOG => '记录',
        ];
        
        $typeName = $typeNames[$this->rule_type] ?? $this->rule_type;
        $actionName = $actionNames[$this->action] ?? $this->action;
        
        return "{$actionName} {$typeName}: {$this->rule_pattern}";
    }
}
