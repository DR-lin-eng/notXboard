<?php

namespace App\Services;

use App\Models\User;
use App\Models\UserGroupLimit;
use App\Models\UserIndividualLimit;
use App\Models\UserOnlineSession;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Collection;

class LimitControlService
{
    /**
     * 设置用户组限制
     */
    public function setGroupLimits(int $trustLevel, array $limits): UserGroupLimit
    {
        return UserGroupLimit::updateOrCreate(
            ['trust_level' => $trustLevel],
            [
                'speed_limit_up' => $limits['speed_limit_up'] ?? 0,
                'speed_limit_down' => $limits['speed_limit_down'] ?? 0,
                'device_limit' => $limits['device_limit'] ?? 0,
                'connection_limit' => $limits['connection_limit'] ?? 0,
            ]
        );
    }
    
    /**
     * 设置用户个人限制
     */
    public function setUserLimits(User $user, array $limits): UserIndividualLimit
    {
        return UserIndividualLimit::updateOrCreate(
            ['user_id' => $user->id],
            [
                'speed_limit_up' => $limits['speed_limit_up'] ?? 0,
                'speed_limit_down' => $limits['speed_limit_down'] ?? 0,
                'device_limit' => $limits['device_limit'] ?? 0,
                'connection_limit' => $limits['connection_limit'] ?? 0,
            ]
        );
    }
    
    /**
     * 获取用户有效限制（个人限制优先于组限制）
     */
    public function getEffectiveLimits(User $user): array
    {
        $individualLimit = $user->relationLoaded('individualLimit')
            ? $user->getRelation('individualLimit')
            : $user->individualLimit;
        $groupLimit = UserGroupLimit::query()
            ->where('trust_level', (int) $user->trust_level)
            ->first();

        return $this->buildEffectiveLimits($individualLimit, $groupLimit);
    }

    public function getEffectiveLimitsForUsers(Collection $users): array
    {
        if ($users->isEmpty()) {
            return [];
        }

        $users->loadMissing('individualLimit');
        $trustLevels = $users->pluck('trust_level')
            ->map(fn ($value) => (int) $value)
            ->unique()
            ->values()
            ->all();

        $groupLimits = UserGroupLimit::query()
            ->whereIn('trust_level', $trustLevels)
            ->get()
            ->keyBy(fn (UserGroupLimit $limit) => (int) $limit->trust_level);

        $result = [];
        foreach ($users as $user) {
            $result[(int) $user->id] = $this->buildEffectiveLimits(
                $user->individualLimit,
                $groupLimits->get((int) $user->trust_level)
            );
        }

        return $result;
    }
    
    /**
     * 获取有效值（个人设置优先，0表示使用组设置）
     */
    private function getEffectiveValue(?int $individualValue, ?int $groupValue, int $defaultValue): int
    {
        if ($individualValue !== null && $individualValue > 0) {
            return $individualValue;
        }
        
        if ($groupValue !== null && $groupValue > 0) {
            return $groupValue;
        }
        
        return $defaultValue;
    }

    private function buildEffectiveLimits(?UserIndividualLimit $individualLimit, ?UserGroupLimit $groupLimit): array
    {
        return [
            'speed_limit_up' => $this->getEffectiveValue(
                $individualLimit?->speed_limit_up,
                $groupLimit?->speed_limit_up,
                0
            ),
            'speed_limit_down' => $this->getEffectiveValue(
                $individualLimit?->speed_limit_down,
                $groupLimit?->speed_limit_down,
                0
            ),
            'device_limit' => $this->getEffectiveValue(
                $individualLimit?->device_limit,
                $groupLimit?->device_limit,
                2
            ),
            'connection_limit' => $this->getEffectiveValue(
                $individualLimit?->connection_limit,
                $groupLimit?->connection_limit,
                10
            ),
        ];
    }
    
    /**
     * 检查设备限制
     */
    public function checkDeviceLimit(User $user, string $ipAddress, int $nodeId): bool
    {
        $limits = $this->getEffectiveLimits($user);
        $deviceLimit = $limits['device_limit'];
        
        if ($deviceLimit <= 0) {
            return true; // 无限制
        }

        $activeIps = UserOnlineSession::where('user_id', $user->id)
            ->where('node_id', $nodeId)
            ->where('last_activity', '>', now()->subMinutes(5)) // 5分钟内活跃
            ->pluck('ip_address')
            ->map(fn ($ip) => (string) $ip)
            ->all();

        $summary = $this->summarizeIpAddresses($activeIps);
        if ($this->isIpAlreadyPresent($summary, $ipAddress)) {
            return true;
        }

        return $summary['device_count'] < $deviceLimit;
    }
    
    /**
     * 检查同用户多IP并发限制
     */
    public function checkCrossNodeIPLimit(User $user, string $ipAddress): bool
    {
        return $this->checkConcurrentIPLimit($user, $ipAddress);
    }
    
    /**
     * 检查同用户多IP并发限制（0 => 默认 3）
     */
    public function checkConcurrentIPLimit(User $user, string $ipAddress): bool
    {
        $limit = (int) ($user->concurrent_ip_limit ?? 0);
        if ($limit <= 0) {
            $limit = 3;
        }

        $activeIps = UserOnlineSession::where('user_id', $user->id)
            ->where('last_activity', '>', now()->subMinutes(5))
            ->pluck('ip_address')
            ->map(fn ($ip) => (string) $ip)
            ->all();

        $summary = $this->summarizeIpAddresses($activeIps);
        if ($this->isIpAlreadyPresent($summary, $ipAddress)) {
            return true;
        }

        return $summary['device_count'] < $limit;
    }
    
    /**
     * 检查连接数限制
     */
    public function checkConnectionLimit(User $user, int $nodeId): bool
    {
        $limits = $this->getEffectiveLimits($user);
        $connectionLimit = $limits['connection_limit'];
        
        if ($connectionLimit <= 0) {
            return true; // 无限制
        }
        
        // 获取当前连接数
        $currentConnections = UserOnlineSession::where('user_id', $user->id)
            ->where('node_id', $nodeId)
            ->where('last_activity', '>', now()->subMinutes(5))
            ->sum('connection_count');
        
        return $currentConnections < $connectionLimit;
    }
    
    /**
     * 获取用户在线IP列表
     */
    public function getOnlineIPs(User $user, int $nodeId): array
    {
        return UserOnlineSession::where('user_id', $user->id)
            ->where('node_id', $nodeId)
            ->where('last_activity', '>', now()->subMinutes(5))
            ->pluck('ip_address')
            ->unique()
            ->values()
            ->toArray();
    }
    
    /**
     * 获取用户全局在线IP列表
     */
    public function getGlobalOnlineIPs(User $user): array
    {
        return UserOnlineSession::where('user_id', $user->id)
            ->where('last_activity', '>', now()->subMinutes(5))
            ->select('ip_address', 'node_id')
            ->get()
            ->groupBy('ip_address')
            ->map(function ($sessions, $ip) {
                return [
                    'ip' => $ip,
                    'nodes' => $sessions->pluck('node_id')->unique()->values()->toArray(),
                    'node_count' => $sessions->pluck('node_id')->unique()->count(),
                ];
            })
            ->values()
            ->toArray();
    }
    
    /**
     * 获取当前连接数
     */
    public function getCurrentConnections(User $user, int $nodeId): int
    {
        return UserOnlineSession::where('user_id', $user->id)
            ->where('node_id', $nodeId)
            ->where('last_activity', '>', now()->subMinutes(5))
            ->sum('connection_count');
    }
    
    /**
     * 记录用户在线会话
     */
    public function recordOnlineSession(User $user, int $nodeId, string $ipAddress, array $sessionData = []): void
    {
        UserOnlineSession::updateOrCreate(
            [
                'user_id' => $user->id,
                'node_id' => $nodeId,
                'ip_address' => $ipAddress,
            ],
            [
                'user_agent' => $sessionData['user_agent'] ?? null,
                'connection_count' => $sessionData['connection_count'] ?? 1,
                'upload_traffic' => $sessionData['upload_traffic'] ?? 0,
                'download_traffic' => $sessionData['download_traffic'] ?? 0,
                'last_activity' => now(),
            ]
        );
    }
    
    /**
     * 清理过期的在线会话
     */
    public function cleanupExpiredSessions(): int
    {
        return UserOnlineSession::where('last_activity', '<', now()->subMinutes(10))
            ->delete();
    }
    
    /**
     * 获取用户组限制配置
     */
    public function getGroupLimits(int $trustLevel): ?UserGroupLimit
    {
        return UserGroupLimit::where('trust_level', $trustLevel)->first();
    }
    
    /**
     * 获取所有用户组限制配置
     */
    public function getAllGroupLimits(): Collection
    {
        return UserGroupLimit::orderBy('trust_level')->get();
    }
    
    /**
     * 删除用户个人限制
     */
    public function removeUserLimits(User $user): bool
    {
        return UserIndividualLimit::where('user_id', $user->id)->delete() > 0;
    }
    
    /**
     * 批量设置用户组限制
     */
    public function batchSetGroupLimits(array $groupLimits): array
    {
        $results = [];
        
        DB::transaction(function () use ($groupLimits, &$results) {
            foreach ($groupLimits as $trustLevel => $limits) {
                if ($trustLevel >= 0 && $trustLevel <= 4) {
                    $results[$trustLevel] = $this->setGroupLimits($trustLevel, $limits);
                }
            }
        });
        
        return $results;
    }
    
    /**
     * 应用限制到用户（当信任等级变化时）
     */
    public function applyLimitsToUser(User $user): void
    {
        // 如果用户没有个人限制，应用组限制到用户模型
        if (!$user->individualLimit) {
            $limits = $this->getEffectiveLimits($user);
            
            $user->update([
                'device_limit' => $limits['device_limit'],
                'speed_limit' => $limits['speed_limit_down'], // 使用下载限速作为主要限速
            ]);
            
            Log::info('Applied group limits to user', [
                'user_id' => $user->id,
                'trust_level' => $user->trust_level,
                'limits' => $limits
            ]);
        }
    }
    
    /**
     * 验证限制配置的有效性
     */
    public function validateLimits(array $limits): array
    {
        $errors = [];
        
        if (isset($limits['speed_limit_up']) && $limits['speed_limit_up'] < 0) {
            $errors[] = 'Upload speed limit cannot be negative';
        }
        
        if (isset($limits['speed_limit_down']) && $limits['speed_limit_down'] < 0) {
            $errors[] = 'Download speed limit cannot be negative';
        }
        
        if (isset($limits['device_limit']) && $limits['device_limit'] < 0) {
            $errors[] = 'Device limit cannot be negative';
        }
        
        if (isset($limits['connection_limit']) && $limits['connection_limit'] < 0) {
            $errors[] = 'Connection limit cannot be negative';
        }
        
        return $errors;
    }

    private function summarizeIpAddresses(array $ipAddresses): array
    {
        $ipv4 = [];
        $ipv6 = [];

        foreach ($ipAddresses as $ipAddress) {
            $ip = trim((string) $ipAddress);
            if ($ip === '') {
                continue;
            }

            if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4)) {
                $ipv4[$ip] = true;
                continue;
            }

            if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV6)) {
                $ipv6[$ip] = true;
            }
        }

        return [
            'ipv4' => array_keys($ipv4),
            'ipv6' => array_keys($ipv6),
            'device_count' => max(count($ipv4), count($ipv6)),
        ];
    }

    private function isIpAlreadyPresent(array $summary, string $ipAddress): bool
    {
        $ip = trim($ipAddress);
        if ($ip === '') {
            return false;
        }

        if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4)) {
            return in_array($ip, $summary['ipv4'] ?? [], true);
        }

        if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV6)) {
            return in_array($ip, $summary['ipv6'] ?? [], true);
        }

        return false;
    }
}
