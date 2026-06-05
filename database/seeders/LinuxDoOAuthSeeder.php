<?php

namespace Database\Seeders;

use App\Models\User;
use App\Services\ApiKeyService;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Log;

class LinuxDoOAuthSeeder extends Seeder
{
    /**
     * 运行数据库种子
     */
    public function run(): void
    {
        Log::info('Starting LinuxDoOAuth seeding');
        
        // 为所有没有 API 密钥的用户生成密钥
        $this->generateMissingApiKeys();
        
        // 创建测试超级管理员用户（如果不存在）
        $this->createTestSuperAdmin();
        
        Log::info('LinuxDoOAuth seeding completed');
    }
    
    /**
     * 为没有 API 密钥的用户生成密钥
     */
    private function generateMissingApiKeys(): void
    {
        $apiKeyService = app(ApiKeyService::class);
        
        $usersWithoutKeys = User::whereNull('api_key')->get();
        
        if ($usersWithoutKeys->isEmpty()) {
            Log::info('All users already have API keys');
            return;
        }
        
        Log::info('Generating API keys for users without keys', [
            'count' => $usersWithoutKeys->count()
        ]);
        
        foreach ($usersWithoutKeys as $user) {
            try {
                $apiKeyService->generateApiKey($user);
                Log::info('Generated API key for user', ['user_id' => $user->id]);
            } catch (\Exception $e) {
                Log::error('Failed to generate API key for user', [
                    'user_id' => $user->id,
                    'error' => $e->getMessage()
                ]);
            }
        }
    }
    
    /**
     * 创建测试超级管理员用户
     */
    private function createTestSuperAdmin(): void
    {
        // 检查是否已有超级管理员
        $existingSuperAdmin = User::where('is_super_admin', true)->first();
        
        if ($existingSuperAdmin) {
            Log::info('Super admin already exists', [
                'user_id' => $existingSuperAdmin->id,
                'email' => $existingSuperAdmin->email
            ]);
            return;
        }
        
        // 仅在开发环境创建测试超级管理员
        if (app()->environment(['local', 'development', 'testing'])) {
            try {
                $superAdmin = User::create([
                    'email' => 'superadmin@example.com',
                    'password' => bcrypt('password'),
                    'token' => \App\Utils\Helper::guid(),
                    'subscribe_path' => \App\Utils\Helper::randomLetters(10),
                    'subscribe_key' => \App\Utils\Helper::randomLetters(8),
                    'subscribe_salt' => \App\Utils\Helper::randomLetters(6),
                    'uuid' => \Illuminate\Support\Str::uuid(),
                    'is_admin' => true,
                    'is_super_admin' => true,
                    'trust_level' => 4,
                    'device_limit' => 50,
                    'transfer_enable' => 1024 * 1024 * 1024 * 100, // 100GB
                ]);
                
                // 生成 API 密钥
                $apiKeyService = app(ApiKeyService::class);
                $apiKeyService->generateApiKey($superAdmin);
                
                Log::info('Test super admin created', [
                    'user_id' => $superAdmin->id,
                    'email' => $superAdmin->email
                ]);
                
            } catch (\Exception $e) {
                Log::error('Failed to create test super admin', [
                    'error' => $e->getMessage()
                ]);
            }
        }
    }
}
