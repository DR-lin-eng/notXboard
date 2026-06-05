<?php
use App\Support\Setting;
use Illuminate\Support\Facades\App;

if (!function_exists('admin_setting')) {
    /**
     * 获取或保存配置参数.
     *
     * @param  string|array  $key
     * @param  mixed  $default
     * @return App\Support\Setting|mixed
     */
    function admin_setting($key = null, $default = null)
    {
        $setting = app(Setting::class);

        if ($key === null) {
            return $setting->toArray();
        }

        if (is_array($key)) {
            $setting->save($key);
            return '';
        }

        $default = config('v2board.' . $key) ?? $default;
        return $setting->get($key) ?? $default;
    }
}

if (!function_exists('admin_settings_batch')) {
    /**
     * 批量获取配置参数，性能优化版本
     *
     * @param array $keys 配置键名数组
     * @return array 返回键值对数组
     */
    function admin_settings_batch(array $keys): array
    {
        return app(Setting::class)->getBatch($keys);
    }
}

if (!function_exists('source_base_url')) {
    /**
     * 获取站点基础 URL，优先使用显式配置，避免信任客户端传入的 Referer
     * @param string $path
     * @return string
     */
    function source_base_url(string $path = ''): string
    {
        $baseUrl = '';
        $configuredBaseUrl = '';
        $configuredUrl = (string) (admin_setting('app_url', config('app.url')) ?: config('app.url'));
        $requestBaseUrl = '';

        if (app()->bound('request')) {
            try {
                $request = request();
                $requestHost = (string) (
                    $request->headers->get('x-forwarded-host')
                    ?: $request->headers->get('host')
                    ?: $request->server('HTTP_HOST', '')
                );
                $requestHost = trim(explode(',', $requestHost)[0] ?? '');
                $requestScheme = (string) (
                    $request->headers->get('x-forwarded-proto')
                    ?: $request->getScheme()
                );
                $requestScheme = trim(explode(',', $requestScheme)[0] ?? '');

                if ($requestHost !== '' && $requestScheme !== '') {
                    $requestBaseUrl = $requestScheme . '://' . $requestHost;
                }
            } catch (\Throwable) {
                $requestBaseUrl = '';
            }
        }

        $isLocalHost = static function (string $host): bool {
            $normalizedHost = strtolower(trim($host, '[]'));

            if ($normalizedHost === '' || $normalizedHost === 'localhost' || str_ends_with($normalizedHost, '.local')) {
                return true;
            }

            if (filter_var($normalizedHost, FILTER_VALIDATE_IP) !== false) {
                if ($normalizedHost === '::1') {
                    return true;
                }

                return !filter_var(
                    $normalizedHost,
                    FILTER_VALIDATE_IP,
                    FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE
                );
            }

            return !str_contains($normalizedHost, '.');
        };

        if ($configuredUrl !== '') {
            $parsedUrl = parse_url($configuredUrl);
            if (isset($parsedUrl['scheme'], $parsedUrl['host'])) {
                $configuredBaseUrl = $parsedUrl['scheme'] . '://' . $parsedUrl['host'];
                if (isset($parsedUrl['port'])) {
                    $configuredBaseUrl .= ':' . $parsedUrl['port'];
                }
                if (!empty($parsedUrl['path']) && $parsedUrl['path'] !== '/') {
                    $configuredBaseUrl .= '/' . ltrim((string) $parsedUrl['path'], '/');
                }

                if (!$requestBaseUrl || !$isLocalHost((string) $parsedUrl['host'])) {
                    $baseUrl = $configuredBaseUrl;
                }
            }
        }

        if (!$baseUrl) {
            $baseUrl = $requestBaseUrl ?: $configuredBaseUrl;
        }

        $baseUrl = rtrim($baseUrl, '/');
        $path = ltrim($path, '/');
        return $baseUrl . '/' . $path;
    }
}
