<?php

namespace App\Services\OAuth;

use App\Models\User;
use App\Models\UserGroupLimit;
use App\Exceptions\OAuthException;
use App\Services\Auth\InviteCodeService;
use App\Services\InvitePlanGrantService;
use App\Utils\Helper;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;
use Exception;

class UserSyncService
{
    /**
     * 从 Linux DO 用户数据同步或创建本地用户
     */
    public function syncFromLinuxDo(array $linuxDoUserData, array $tokenData, ?string $inviteCode = null): User
    {
        try {
            DB::beginTransaction();
            
            // 查找现有用户
            $user = User::where('linux_do_id', $linuxDoUserData['id'])->first();
            
            if (!$user) {
                // 创建新用户
                $inviteCodeService = app(InviteCodeService::class);
                $inviteCodeModel = $inviteCodeService->resolveRegistrationInvite($inviteCode, true);

                $user = $this->createNewUser(
                    linuxDoUserData: $linuxDoUserData,
                    tokenData: $tokenData,
                    inviteUserId: $inviteCodeModel?->user_id
                );
                $inviteCodeService->consumeInviteCode($inviteCodeModel);
                app(InvitePlanGrantService::class)->grantFromInviteCode($user, $inviteCodeModel);
                Log::info('Created new Linux DO user', ['linux_do_id' => $linuxDoUserData['id']]);
            } else {
                // 更新现有用户
                $user = $this->updateExistingUser($user, $linuxDoUserData, $tokenData);
                Log::info('Updated existing Linux DO user', ['user_id' => $user->id]);
            }
            
            DB::commit();
            return $user;
            
        } catch (Exception $e) {
            DB::rollBack();
            Log::error('User sync failed', [
                'error' => $e->getMessage(),
                'linux_do_data' => $linuxDoUserData
            ]);
            throw $e;
        }
    }
    
    /**
     * 创建新用户
     */
    private function createNewUser(array $linuxDoUserData, array $tokenData, ?int $inviteUserId = null): User
    {
        // 生成唯一的邮箱（如果 Linux DO 没有提供）
        $email = $linuxDoUserData['email'] ?? $linuxDoUserData['username'] . '@linux.do';
        
        // 确保邮箱唯一
        $originalEmail = $email;
        $counter = 1;
        while (User::where('email', $email)->exists()) {
            $email = str_replace('@', "+{$counter}@", $originalEmail);
            $counter++;
        }
        
        $user = User::create([
            'email' => $email,
            'password' => bcrypt(Str::random(32)), // 随机密码，用户通过 OAuth 登录
            'token' => $this->generateUniqueToken(),
            'uuid' => Helper::guid(true),
            'invite_user_id' => $inviteUserId,
            'subscribe_path' => Helper::randomLetters(10),
            'subscribe_key' => Helper::randomLetters(8),
            'subscribe_salt' => Helper::randomLetters(6),
            
            // Linux DO Connect 相关字段
            'linux_do_id' => $linuxDoUserData['id'],
            'linux_do_username' => $linuxDoUserData['username'],
            'linux_do_name' => $linuxDoUserData['name'],
            'linux_do_avatar' => $linuxDoUserData['avatar_template'],
            'trust_level' => $linuxDoUserData['trust_level'],
            'is_silenced' => $linuxDoUserData['silenced'],
            'external_ids' => $linuxDoUserData['external_ids'],
            
            // OAuth 令牌信息
            'oauth_provider' => 'linux_do',
            'oauth_access_token' => $tokenData['access_token'],
            'oauth_refresh_token' => $tokenData['refresh_token'] ?? null,
            'oauth_expires_at' => now()->addSeconds($tokenData['expires_in']),
            
            // 默认设置
            'banned' => !$linuxDoUserData['active'],
            'commission_rate' => 0.1,
            'commission_type' => User::COMMISSION_TYPE_SYSTEM,
            'device_limit' => $this->getDefaultDeviceLimit($linuxDoUserData['trust_level']),
            'last_login_at' => time(),
        ]);

        if ($user->subscribe_salt === $user->subscribe_key) {
            $user->subscribe_salt = Helper::randomLetters(6);
            $user->save();
        }
        
        // 生成 API 密钥
        $user->generateApiKey();
        
        // 根据信任等级分配用户组
        $this->updateUserGroup($user);
        
        return $user;
    }
    
    /**
     * 更新现有用户
     */
    private function updateExistingUser(User $user, array $linuxDoUserData, array $tokenData): User
    {
        $oldTrustLevel = $user->trust_level;
        
        $user->update([
            'linux_do_username' => $linuxDoUserData['username'],
            'linux_do_name' => $linuxDoUserData['name'],
            'linux_do_avatar' => $linuxDoUserData['avatar_template'],
            'trust_level' => $linuxDoUserData['trust_level'],
            'is_silenced' => $linuxDoUserData['silenced'],
            'external_ids' => $linuxDoUserData['external_ids'],
            
            // 更新 OAuth 令牌
            'oauth_access_token' => $tokenData['access_token'],
            'oauth_refresh_token' => $tokenData['refresh_token'] ?? $user->oauth_refresh_token,
            'oauth_expires_at' => now()->addSeconds($tokenData['expires_in']),
            
            // 更新状态
            'banned' => !$linuxDoUserData['active'],
            'last_login_at' => time(),
        ]);
        
        // 如果信任等级发生变化，更新用户组
        if ($oldTrustLevel !== $user->trust_level) {
            $this->updateUserGroup($user);
            Log::info('User trust level changed', [
                'user_id' => $user->id,
                'old_level' => $oldTrustLevel,
                'new_level' => $user->trust_level
            ]);
        }
        
        return $user;
    }
    
    /**
     * 根据信任等级更新用户组
     */
    public function updateUserGroup(User $user): void
    {
        // 根据信任等级获取对应的用户组限制
        $groupLimit = UserGroupLimit::where('trust_level', $user->trust_level)->first();
        
        if ($groupLimit) {
            // 如果用户没有个人限制，应用组限制
            if (!$user->individualLimit) {
                $user->update([
                    'device_limit' => $groupLimit->device_limit ?: $user->device_limit,
                    'speed_limit' => $groupLimit->speed_limit_down ?: $user->speed_limit,
                ]);
            }
        }
        
        Log::info('Updated user group for trust level', [
            'user_id' => $user->id,
            'trust_level' => $user->trust_level,
            'group' => $user->getUserGroup()
        ]);
    }
    
    /**
     * 生成唯一的用户令牌
     */
    private function generateUniqueToken(): string
    {
        do {
            $token = Helper::guid();
        } while (User::where('token', $token)->exists());
        
        return $token;
    }
    
    /**
     * 根据信任等级获取默认设备限制
     */
    private function getDefaultDeviceLimit(int $trustLevel): int
    {
        return match($trustLevel) {
            0 => 2,  // 新用户
            1 => 3,  // 基础用户
            2 => 5,  // 成员
            3 => 8,  // 常规用户
            4 => 10, // 领导者
            default => 2
        };
    }
    
    /**
     * 检查用户是否需要更新信息
     */
    public function shouldUpdateUserInfo(User $user): bool
    {
        if (!$user->isLinuxDoUser()) {
            return false;
        }
        
        // 如果令牌即将过期（1小时内），需要刷新
        if ($user->oauth_expires_at) {
            $expiresAt = $user->oauth_expires_at instanceof Carbon
                ? $user->oauth_expires_at->copy()
                : Carbon::parse($user->oauth_expires_at);

            if ($expiresAt->subHour()->isPast()) {
                return true;
            }
        }

        if ($user->updated_at === null) {
            return true;
        }
        
        // 如果超过24小时没有同步，需要更新
        $updatedAt = $user->updated_at instanceof Carbon
            ? $user->updated_at
            : Carbon::createFromTimestamp((int) $user->updated_at);

        return $updatedAt->lt(now()->subDay());
    }
    
    /**
     * 刷新用户的 OAuth 令牌
     */
    public function refreshUserToken(User $user, LinuxDoOAuthService $oauthService): bool
    {
        if (!$user->oauth_refresh_token) {
            Log::warning('No refresh token available for user', ['user_id' => $user->id]);
            return false;
        }
        
        try {
            $tokenData = $oauthService->refreshToken($user->oauth_refresh_token);
            
            $user->update([
                'oauth_access_token' => $tokenData['access_token'],
                'oauth_refresh_token' => $tokenData['refresh_token'],
                'oauth_expires_at' => now()->addSeconds($tokenData['expires_in']),
            ]);
            
            Log::info('Refreshed OAuth token for user', ['user_id' => $user->id]);
            return true;
            
        } catch (Exception $e) {
            Log::error('Failed to refresh OAuth token', [
                'user_id' => $user->id,
                'error' => $e->getMessage()
            ]);
            return false;
        }
    }
    
    /**
     * 同步用户最新信息
     */
    public function syncUserInfo(User $user, LinuxDoOAuthService $oauthService): bool
    {
        try {
            // 如果令牌过期，先尝试刷新
            if ($user->isOAuthTokenExpired()) {
                if (!$this->refreshUserToken($user, $oauthService)) {
                    return false;
                }
            }
            
            // 获取最新用户信息
            $userInfo = $oauthService->getUserInfo($user->oauth_access_token);
            
            // 更新用户信息（不包含令牌）
            $oldTrustLevel = $user->trust_level;
            
            $user->update([
                'linux_do_username' => $userInfo['username'],
                'linux_do_name' => $userInfo['name'],
                'linux_do_avatar' => $userInfo['avatar_template'],
                'trust_level' => $userInfo['trust_level'],
                'is_silenced' => $userInfo['silenced'],
                'external_ids' => $userInfo['external_ids'],
                'banned' => !$userInfo['active'],
            ]);
            
            // 如果信任等级变化，更新用户组
            if ($oldTrustLevel !== $user->trust_level) {
                $this->updateUserGroup($user);
            }
            
            Log::info('Synced user info from Linux DO', ['user_id' => $user->id]);
            return true;
            
        } catch (Exception $e) {
            Log::error('Failed to sync user info', [
                'user_id' => $user->id,
                'error' => $e->getMessage()
            ]);
            return false;
        }
    }
}
