<?php

namespace App\Http\Controllers\V1\Passport;

use App\Helpers\ResponseEnum;
use App\Http\Controllers\Controller;
use App\Http\Requests\Passport\AuthForget;
use App\Http\Requests\Passport\AuthLogin;
use App\Http\Requests\Passport\AuthRegister;
use App\Models\User;
use App\Services\OAuth\LinuxDoOAuthService;
use App\Services\OAuth\UserSyncService;
use App\Services\Auth\LoginService;
use App\Services\Auth\MailLinkService;
use App\Services\Auth\RegisterModeService;
use App\Services\Auth\RegisterService;
use App\Services\AuthService;
use App\Services\CaptchaService;
use App\Services\PowService;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;

class AuthController extends Controller
{
    protected MailLinkService $mailLinkService;
    protected RegisterService $registerService;
    protected LoginService $loginService;

    public function __construct(
        MailLinkService $mailLinkService,
        RegisterService $registerService,
        LoginService $loginService
    ) {
        $this->mailLinkService = $mailLinkService;
        $this->registerService = $registerService;
        $this->loginService = $loginService;
    }

    /**
     * 通过邮件链接登录
     */
    public function loginWithMailLink(Request $request)
    {
        if (config('ops.telegram_only_mode')) {
            return $this->fail([403, __('Email login is disabled in Telegram-only mode')]);
        }

        $params = $request->validate([
            'email' => 'required|email:strict',
            'redirect' => 'nullable'
        ]);
        $user = User::where('email', $params['email'])->first();
        if (!$this->canUseEmailLogin($user)) {
            return $this->fail([403, __('Email login is disabled. Please use OAuth2 login')]);
        }

        [$success, $result] = $this->mailLinkService->handleMailLink(
            $params['email'],
            $request->input('redirect')
        );

        if (!$success) {
            return $this->fail($result);
        }

        return $this->success($result);
    }

    /**
     * 用户注册
     */
    public function register(AuthRegister $request)
    {
        if (!app(RegisterModeService::class)->allowsEmailRegistration()) {
            return $this->fail([403, __('Registration by email is disabled')]);
        }

        [$success, $result] = $this->registerService->register($request);

        if (!$success) {
            return $this->fail($result);
        }

        $authService = new AuthService($result);
        return $this->success($authService->generateAuthData());
    }

    /**
     * 用户登录
     */
    public function login(AuthLogin $request)
    {
        $captchaService = app(CaptchaService::class);
        [$captchaValid, $captchaError] = $captchaService->verify($request);
        if (!$captchaValid) {
            return $this->fail($captchaError);
        }

        $powService = app(PowService::class);
        [$powValid, $powError] = $powService->verify($request);
        if (!$powValid) {
            return $this->fail($powError);
        }

        $email = $request->input('email');
        $password = $request->input('password');

        [$success, $result] = $this->loginService->login($email, $password);

        if (!$success) {
            return $this->fail($result);
        }

        if ($result instanceof User) {
            $this->syncLinuxDoUserTrustLevelOnLogin($result);
        }

        $authService = new AuthService($result);
        return $this->success($authService->generateAuthData());
    }

    /**
     * 获取登录 PoW 挑战
     */
    public function powChallenge(Request $request)
    {
        $powService = app(PowService::class);
        [$success, $result] = $powService->generateChallenge($request);

        if (!$success) {
            return $this->fail($result);
        }

        return $this->success($result);
    }

    /**
     * 通过token登录
     */
    public function token2Login(Request $request)
    {
        // 处理直接通过token重定向
        if ($token = $request->input('token')) {
            $redirect = '/app/#/login?verify=' . $token . '&redirect=' . ($request->input('redirect', 'dashboard'));

            return redirect()->to(
                admin_setting('app_url')
                ? admin_setting('app_url') . $redirect
                : url($redirect)
            );
        }

        // 处理通过验证码登录
        if ($verify = $request->input('verify')) {
            $userId = $this->mailLinkService->handleTokenLogin($verify);

            if (!$userId) {
                return response()->json([
                    'message' => __('Token error')
                ], 400);
            }

            $user = \App\Models\User::find($userId);

            if (!$user) {
                return response()->json([
                    'message' => __('User not found')
                ], 400);
            }
            if ($user->banned) {
                return response()->json([
                    'message' => $user->getSuspensionMessage()
                ], 403);
            }
            if (!$this->canUseEmailLogin($user)) {
                return response()->json([
                    'message' => __('Email login is disabled. Please use OAuth2 login')
                ], 403);
            }

            $this->syncLinuxDoUserTrustLevelOnLogin($user);

            $authService = new AuthService($user);

            return response()->json([
                'data' => $authService->generateAuthData()
            ]);
        }

        return response()->json([
            'message' => __('Invalid request')
        ], 400);
    }

    /**
     * 获取快速登录URL
     */
    public function getQuickLoginUrl(Request $request)
    {
        $authorization = $request->input('auth_data') ?? $request->header('authorization');

        if (!$authorization) {
            return response()->json([
                'message' => ResponseEnum::CLIENT_HTTP_UNAUTHORIZED
            ], 401);
        }

        $user = AuthService::findUserByBearerToken($authorization);

        if (!$user) {
            return response()->json([
                'message' => ResponseEnum::CLIENT_HTTP_UNAUTHORIZED_EXPIRED
            ], 401);
        }

        $url = $this->loginService->generateQuickLoginUrl($user, $request->input('redirect'));
        return $this->success($url);
    }

    /**
     * 忘记密码处理
     */
    public function forget(AuthForget $request)
    {
        if (config('ops.telegram_only_mode')) {
            return $this->fail([403, __('Password reset by email is disabled in Telegram-only mode')]);
        }

        $user = User::where('email', (string) $request->input('email'))->first();
        if (!$this->canUseEmailLogin($user)) {
            return $this->fail([403, __('Email login is disabled. Please use OAuth2 login')]);
        }

        [$success, $result] = $this->loginService->resetPassword(
            $request->input('email'),
            $request->input('email_code'),
            $request->input('password')
        );

        if (!$success) {
            return $this->fail($result);
        }

        return $this->success(true);
    }

    private function isForceOauth2LoginEnabled(): bool
    {
        return (bool) admin_setting('force_oauth2_login', 0);
    }

    private function canUseEmailLogin(?User $user): bool
    {
        return app(RegisterModeService::class)->allowsEmailLogin($user);
    }

    private function syncLinuxDoUserTrustLevelOnLogin(User $user): void
    {
        if (!$user->isLinuxDoUser()) {
            return;
        }

        try {
            $syncService = app(UserSyncService::class);
            $oauthService = app(LinuxDoOAuthService::class);
            $syncService->syncUserInfo($user, $oauthService);
        } catch (\Throwable $e) {
            Log::warning('Login trust-level sync failed', [
                'user_id' => $user->id,
                'error' => $e->getMessage(),
            ]);
        }
    }
}
