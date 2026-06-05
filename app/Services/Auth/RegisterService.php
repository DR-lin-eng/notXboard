<?php

namespace App\Services\Auth;

use App\Models\InviteCode;
use App\Models\User;
use App\Services\CaptchaService;
use App\Services\InvitePlanGrantService;
use App\Services\Plugin\HookManager;
use App\Services\UserService;
use App\Utils\CacheKey;
use App\Utils\Dict;
use App\Utils\Helper;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;
use App\Services\PowService;

class RegisterService
{
    /**
     * 验证用户注册请求
     *
     * @param Request $request 请求对象
     * @return array [是否通过, 错误消息]
     */
    public function validateRegister(Request $request): array
    {
        $registerModeService = app(RegisterModeService::class);
        if (!$registerModeService->allowsEmailRegistration()) {
            return [false, [403, __('Registration by email is disabled')]];
        }

        // 检查IP注册限制
        if ((int) admin_setting('register_limit_by_ip_enable', 0)) {
            $registerCountByIP = Cache::get(CacheKey::get('REGISTER_IP_RATE_LIMIT', $request->ip())) ?? 0;
            if ((int) $registerCountByIP >= (int) admin_setting('register_limit_count', 3)) {
                return [
                    false,
                    [
                        429,
                        __('Register frequently, please try again after :minute minute', [
                            'minute' => admin_setting('register_limit_expire', 60)
                        ])
                    ]
                ];
            }
        }

        // 检查验证码
        $captchaService = app(CaptchaService::class);
        [$captchaValid, $captchaError] = $captchaService->verify($request);
        if (!$captchaValid) {
            return [false, $captchaError];
        }

        $powService = app(PowService::class);
        [$powValid, $powError] = $powService->verify($request);
        if (!$powValid) {
            return [false, $powError];
        }

        // 检查邮箱白名单
        if ((int) admin_setting('email_whitelist_enable', 0)) {
            if (
                !Helper::emailSuffixVerify(
                    $request->input('email'),
                    admin_setting('email_whitelist_suffix', Dict::EMAIL_WHITELIST_SUFFIX_DEFAULT)
                )
            ) {
                return [false, [400, __('Email suffix is not in the Whitelist')]];
            }
        }

        // 检查Gmail限制
        if ((int) admin_setting('email_gmail_limit_enable', 0)) {
            $prefix = explode('@', $request->input('email'))[0];
            if (strpos($prefix, '.') !== false || strpos($prefix, '+') !== false) {
                return [false, [400, __('Gmail alias is not supported')]];
            }
        }

        // 检查邀请码要求
        if ((int) admin_setting('invite_force', 0)) {
            if (empty($request->input('invite_code'))) {
                return [false, [422, __('You must use the invitation code to register')]];
            }
        }

        // 检查邮箱验证
        if ((int) admin_setting('email_verify', 0)) {
            if (empty($request->input('email_code'))) {
                return [false, [422, __('Email verification code cannot be empty')]];
            }
            if ((string) Cache::get(CacheKey::get('EMAIL_VERIFY_CODE', $request->input('email'))) !== (string) $request->input('email_code')) {
                return [false, [400, __('Incorrect email verification code')]];
            }
        }

        // 检查邮箱是否存在
        $email = $request->input('email');
        $exist = User::where('email', $email)->first();
        if ($exist) {
            return [false, [400201, __('Email already exists')]];
        }

        return [true, null];
    }

    /**
     * 处理邀请码
     *
     * @param string $inviteCode 邀请码
     * @return int|null 邀请人ID
     */
    public function handleInviteCode(string $inviteCode): int|null
    {
        $inviteCodeModel = app(InviteCodeService::class)->resolveRegistrationInvite($inviteCode, false);
        return $inviteCodeModel?->user_id;
    }



    /**
     * 注册用户
     *
     * @param Request $request 请求对象
     * @return array [成功状态, 用户对象或错误信息]
     */
    public function register(Request $request): array
    {
        // 验证注册数据
        [$valid, $error] = $this->validateRegister($request);
        if (!$valid) {
            return [false, $error];
        }

        try {
            $user = DB::transaction(function () use ($request) {
                HookManager::call('user.register.before', $request);

                $email = (string) $request->input('email');
                $password = (string) $request->input('password');
                $inviteCode = (string) $request->input('invite_code', '');

                $inviteCodeService = app(InviteCodeService::class);
                $inviteCodeModel = $inviteCodeService->resolveRegistrationInvite($inviteCode, true);
                $inviteUserId = $inviteCodeModel?->user_id;

                $userService = app(UserService::class);
                $user = $userService->createUser([
                    'email' => $email,
                    'password' => $password,
                    'invite_user_id' => $inviteUserId,
                ]);

                if (!$user->save()) {
                    throw new \RuntimeException(__('Register failed'));
                }

                $inviteCodeService->consumeInviteCode($inviteCodeModel);
                app(InvitePlanGrantService::class)->grantFromInviteCode($user, $inviteCodeModel);

                HookManager::call('user.register.after', $user);

                if ((int) admin_setting('email_verify', 0)) {
                    Cache::forget(CacheKey::get('EMAIL_VERIFY_CODE', $email));
                }

                $user->last_login_at = time();
                $user->save();

                return $user;
            });
        } catch (\Throwable $e) {
            return [false, [$e instanceof \App\Exceptions\ApiException ? ($e->getCode() ?: 422) : 500, $e->getMessage() ?: __('Register failed')]];
        }

        if ((int) admin_setting('register_limit_by_ip_enable', 0)) {
            $registerCountByIP = Cache::get(CacheKey::get('REGISTER_IP_RATE_LIMIT', $request->ip())) ?? 0;
            Cache::put(
                CacheKey::get('REGISTER_IP_RATE_LIMIT', $request->ip()),
                (int) $registerCountByIP + 1,
                (int) admin_setting('register_limit_expire', 60) * 60
            );
        }

        return [true, $user];
    }
}
