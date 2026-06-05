<?php

namespace App\Exceptions;

use Exception;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class OAuthException extends Exception
{
    protected string $redirectUrl;
    
    public function __construct(string $message = "", int $code = 0, string $redirectUrl = '/login', ?Exception $previous = null)
    {
        parent::__construct($message, $code, $previous);
        $this->redirectUrl = $redirectUrl;
    }
    
    /**
     * 渲染异常响应
     */
    public function render(Request $request): JsonResponse
    {
        return response()->json([
            'error' => $this->getMessage(),
            'error_code' => $this->getCode(),
            'redirect_url' => $this->redirectUrl,
            'success' => false,
        ], $this->getStatusCode());
    }
    
    /**
     * 获取 HTTP 状态码
     */
    private function getStatusCode(): int
    {
        return match($this->getCode()) {
            401 => 401, // 未授权
            403 => 403, // 禁止访问
            404 => 404, // 未找到
            422 => 422, // 验证失败
            429 => 429, // 请求过多
            500 => 500, // 服务器错误
            default => 400, // 默认客户端错误
        };
    }
    
    /**
     * 创建授权失败异常
     */
    public static function authorizationFailed(string $reason = 'Authorization failed'): self
    {
        return new self($reason, 401, '/login');
    }
    
    /**
     * 创建令牌交换失败异常
     */
    public static function tokenExchangeFailed(string $reason = 'Token exchange failed'): self
    {
        return new self($reason, 400, '/login');
    }
    
    /**
     * 创建用户信息获取失败异常
     */
    public static function userInfoFailed(string $reason = 'Failed to get user info'): self
    {
        return new self($reason, 400, '/login');
    }
    
    /**
     * 创建令牌刷新失败异常
     */
    public static function tokenRefreshFailed(string $reason = 'Token refresh failed'): self
    {
        return new self($reason, 401, '/login');
    }
    
    /**
     * 创建配置错误异常
     */
    public static function configurationError(string $reason = 'OAuth configuration error'): self
    {
        return new self($reason, 500, '/login');
    }
}
