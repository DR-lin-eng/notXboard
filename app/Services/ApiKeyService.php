<?php

namespace App\Services;

use App\Models\User;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

class ApiKeyService
{
    /**
     * 为用户生成新的 API 密钥
     */
    public function generateApiKey(User $user): string
    {
        do {
            // 生成格式：xb_ + 60位随机字符
            $apiKey = 'xb_' . bin2hex(random_bytes(30));
        } while (User::where('api_key', $apiKey)->exists());
        
        $user->api_key = $apiKey;
        $user->save();
        
        Log::info('API key generated for user', [
            'user_id' => $user->id,
            'key_prefix' => substr($apiKey, 0, 10) . '...'
        ]);
        
        return $apiKey;
    }
    
    /**
     * 重置用户的 API 密钥
     */
    public function resetApiKey(User $user): string
    {
        $oldKeyPrefix = $user->api_key ? substr($user->api_key, 0, 10) . '...' : 'none';
        
        $newApiKey = $this->generateApiKey($user);
        
        Log::info('API key reset for user', [
            'user_id' => $user->id,
            'old_key_prefix' => $oldKeyPrefix,
            'new_key_prefix' => substr($newApiKey, 0, 10) . '...'
        ]);
        
        return $newApiKey;
    }
    
    /**
     * 验证 API 密钥格式
     */
    public function validateApiKeyFormat(string $apiKey): bool
    {
        // 检查格式：xb_ + 60位十六进制字符
        return preg_match('/^xb_[a-f0-9]{60}$/', $apiKey) === 1;
    }
    
    /**
     * 通过 API 密钥查找用户
     */
    public function findUserByApiKey(string $apiKey): ?User
    {
        if (!$this->validateApiKeyFormat($apiKey)) {
            return null;
        }
        
        return User::where('api_key', $apiKey)->first();
    }
    
    /**
     * 验证 API 密钥是否有效
     */
    public function validateApiKey(string $apiKey): bool
    {
        $user = $this->findUserByApiKey($apiKey);
        
        if (!$user) {
            return false;
        }
        
        // 检查用户是否被禁用
        if ($user->banned) {
            Log::warning('API key validation failed - user banned', [
                'user_id' => $user->id,
                'key_prefix' => substr($apiKey, 0, 10) . '...'
            ]);
            return false;
        }
        
        return true;
    }

    public function validateApiKeyForUser(User $user, string $apiKey): bool
    {
        if (!$this->validateApiKeyFormat($apiKey) || empty($user->api_key) || $user->banned) {
            return false;
        }

        return hash_equals((string) $user->api_key, $apiKey);
    }
    
    /**
     * 获取 API 密钥使用统计
     */
    public function getApiKeyStats(User $user): array
    {
        return [
            'user_id' => $user->id,
            'api_key_exists' => !empty($user->api_key),
            'api_key_prefix' => $user->api_key ? substr($user->api_key, 0, 10) . '...' : null,
            'created_at' => $user->created_at,
            'last_login_at' => $user->last_login_at ? date('Y-m-d H:i:s', $user->last_login_at) : null,
            'is_active' => $user->isActive(),
        ];
    }
    
    /**
     * 批量生成 API 密钥（为没有密钥的用户）
     */
    public function batchGenerateApiKeys(): array
    {
        $usersWithoutKeys = User::whereNull('api_key')->get();
        $results = [];
        
        foreach ($usersWithoutKeys as $user) {
            try {
                $apiKey = $this->generateApiKey($user);
                $results[] = [
                    'user_id' => $user->id,
                    'email' => $user->email,
                    'api_key_generated' => true,
                    'api_key_prefix' => substr($apiKey, 0, 10) . '...'
                ];
            } catch (\Exception $e) {
                $results[] = [
                    'user_id' => $user->id,
                    'email' => $user->email,
                    'api_key_generated' => false,
                    'error' => $e->getMessage()
                ];
                
                Log::error('Failed to generate API key for user', [
                    'user_id' => $user->id,
                    'error' => $e->getMessage()
                ]);
            }
        }
        
        Log::info('Batch API key generation completed', [
            'total_users' => count($usersWithoutKeys),
            'successful' => count(array_filter($results, fn($r) => $r['api_key_generated'])),
            'failed' => count(array_filter($results, fn($r) => !$r['api_key_generated']))
        ]);
        
        return $results;
    }
    
    /**
     * 检查 API 密钥是否唯一
     */
    public function isApiKeyUnique(string $apiKey): bool
    {
        return !User::where('api_key', $apiKey)->exists();
    }
    
    /**
     * 生成安全的随机 API 密钥
     */
    public function generateSecureApiKey(): string
    {
        do {
            $apiKey = 'xb_' . bin2hex(random_bytes(30));
        } while (!$this->isApiKeyUnique($apiKey));
        
        return $apiKey;
    }
    
    /**
     * 清理无效的 API 密钥
     */
    public function cleanupInvalidApiKeys(): array
    {
        $results = [
            'invalid_format_count' => 0,
            'duplicate_count' => 0,
            'cleaned_users' => []
        ];
        
        // 查找格式无效的 API 密钥
        $usersWithInvalidKeys = User::whereNotNull('api_key')
            ->get()
            ->filter(function ($user) {
                return !$this->validateApiKeyFormat($user->api_key);
            });
        
        foreach ($usersWithInvalidKeys as $user) {
            $oldKey = $user->api_key;
            $newKey = $this->generateApiKey($user);
            
            $results['invalid_format_count']++;
            $results['cleaned_users'][] = [
                'user_id' => $user->id,
                'email' => $user->email,
                'reason' => 'invalid_format',
                'old_key_prefix' => substr($oldKey, 0, 10) . '...',
                'new_key_prefix' => substr($newKey, 0, 10) . '...'
            ];
        }
        
        // 查找重复的 API 密钥
        $duplicateKeys = User::whereNotNull('api_key')
            ->selectRaw('api_key, COUNT(*) as count')
            ->groupBy('api_key')
            ->having('count', '>', 1)
            ->pluck('api_key');
        
        foreach ($duplicateKeys as $duplicateKey) {
            $users = User::where('api_key', $duplicateKey)->get();
            
            // 保留第一个用户的密钥，为其他用户生成新密钥
            foreach ($users->skip(1) as $user) {
                $newKey = $this->generateApiKey($user);
                
                $results['duplicate_count']++;
                $results['cleaned_users'][] = [
                    'user_id' => $user->id,
                    'email' => $user->email,
                    'reason' => 'duplicate',
                    'old_key_prefix' => substr($duplicateKey, 0, 10) . '...',
                    'new_key_prefix' => substr($newKey, 0, 10) . '...'
                ];
            }
        }
        
        Log::info('API key cleanup completed', $results);
        
        return $results;
    }
    
    /**
     * 获取系统 API 密钥统计信息
     */
    public function getSystemStats(): array
    {
        $totalUsers = User::count();
        $usersWithKeys = User::whereNotNull('api_key')->count();
        $usersWithoutKeys = $totalUsers - $usersWithKeys;
        
        return [
            'total_users' => $totalUsers,
            'users_with_api_keys' => $usersWithKeys,
            'users_without_api_keys' => $usersWithoutKeys,
            'api_key_coverage_percentage' => $totalUsers > 0 ? round(($usersWithKeys / $totalUsers) * 100, 2) : 0,
        ];
    }
}
