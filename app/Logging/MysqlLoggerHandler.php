<?php
namespace App\Logging;

use App\Models\Log as LogModel;
use Illuminate\Http\Request;
use Monolog\Handler\AbstractProcessingHandler;
use Monolog\Level;
use Monolog\LogRecord;
use Throwable;

class MysqlLoggerHandler extends AbstractProcessingHandler
{
    private const MAX_FIELD_LENGTH = 8192;
    private const MAX_DEPTH = 4;
    private const MAX_ARRAY_ITEMS = 60;

    private const SENSITIVE_KEYS = [
        'password',
        'pwd',
        'pass',
        'token',
        'access_token',
        'refresh_token',
        'secret',
        'authorization',
        'cookie',
        'set-cookie',
        'api_key',
        'apikey',
    ];

    public function __construct(int|string|Level $level = Level::Warning, bool $bubble = false)
    {
        parent::__construct($level, $bubble);
    }

    protected function write(LogRecord $record): void
    {
        $request = $this->resolveRequest();
        $timestamp = $record->datetime->getTimestamp();

        $payload = [
            'title' => $this->truncate((string) $record->message),
            'level' => $record->level->getName(),
            'host' => $this->resolveHost($record, $request),
            'uri' => $this->resolveUri($record, $request),
            'method' => $this->resolveMethod($record, $request),
            'ip' => $this->resolveIp($request),
            'data' => $this->encodePayload($this->buildSafeRequestData($request)),
            'context' => $this->encodePayload($this->sanitizeValue($record->context)),
            'created_at' => $timestamp,
            'updated_at' => $timestamp,
        ];

        try {
            LogModel::insert($payload);
        } catch (Throwable $exception) {
            $this->fallbackToStderr($payload, $exception);
        }
    }

    private function resolveRequest(): ?Request
    {
        if (!app()->bound('request')) {
            return null;
        }

        $request = app('request');
        return $request instanceof Request ? $request : null;
    }

    private function resolveHost(LogRecord $record, ?Request $request): string
    {
        $host = $record->extra['request_host'] ?? null;
        if (is_string($host) && $host !== '') {
            return $this->truncate($host, 1024);
        }

        if ($request) {
            return $this->truncate((string) $request->getSchemeAndHttpHost(), 1024);
        }

        return 'cli';
    }

    private function resolveUri(LogRecord $record, ?Request $request): string
    {
        $uri = $record->extra['request_uri'] ?? null;
        if (is_string($uri) && $uri !== '') {
            return $this->truncate($uri, 2048);
        }

        if ($request) {
            return $this->truncate((string) $request->getRequestUri(), 2048);
        }

        return 'cli://artisan';
    }

    private function resolveMethod(LogRecord $record, ?Request $request): string
    {
        $method = $record->extra['request_method'] ?? null;
        if (is_string($method) && $method !== '') {
            return strtoupper($this->truncate($method, 16));
        }

        if ($request) {
            return strtoupper($this->truncate((string) $request->getMethod(), 16));
        }

        return 'CLI';
    }

    private function resolveIp(?Request $request): ?string
    {
        if (!$request) {
            return null;
        }

        $ip = $request->getClientIp();
        return is_string($ip) && $ip !== '' ? $this->truncate($ip, 128) : null;
    }

    private function buildSafeRequestData(?Request $request): array
    {
        if (!$request) {
            return [
                'type' => 'cli',
                'argv' => array_slice((array) ($_SERVER['argv'] ?? []), 0, 6),
            ];
        }

        $route = $request->route();
        $routeUri = is_object($route) && method_exists($route, 'uri') ? $route->uri() : null;

        return [
            'type' => 'http',
            'query_keys' => array_slice(array_keys($request->query()), 0, self::MAX_ARRAY_ITEMS),
            'content_type' => $this->truncate((string) ($request->header('Content-Type') ?? ''), 256),
            'content_length' => (int) ($request->server('CONTENT_LENGTH') ?? 0),
            'route' => $routeUri ? $this->truncate((string) $routeUri, 512) : null,
            'user_agent' => $this->truncate((string) $request->userAgent(), 512),
        ];
    }

    private function sanitizeValue(mixed $value, int $depth = 0, ?string $key = null): mixed
    {
        if ($depth >= self::MAX_DEPTH) {
            return '[depth_limited]';
        }

        if (is_string($key) && $this->isSensitiveKey($key)) {
            return '[redacted]';
        }

        if ($value instanceof Throwable) {
            return [
                'type' => $value::class,
                'message' => $this->truncate($value->getMessage(), 1024),
                'code' => $value->getCode(),
                'file' => $this->truncate($value->getFile(), 512),
                'line' => $value->getLine(),
            ];
        }

        if (is_array($value)) {
            $sanitized = [];
            $count = 0;

            foreach ($value as $childKey => $childValue) {
                if (++$count > self::MAX_ARRAY_ITEMS) {
                    $sanitized['__truncated__'] = sprintf('Only first %d items kept', self::MAX_ARRAY_ITEMS);
                    break;
                }

                $normalizedKey = is_string($childKey) ? $childKey : (string) $childKey;
                $sanitized[$normalizedKey] = $this->sanitizeValue($childValue, $depth + 1, $normalizedKey);
            }

            return $sanitized;
        }

        if (is_object($value)) {
            if (method_exists($value, '__toString')) {
                return $this->truncate((string) $value, 1024);
            }

            return [
                'type' => $value::class,
            ];
        }

        if (is_string($value)) {
            return $this->truncate($value, 2048);
        }

        if (is_scalar($value) || $value === null) {
            return $value;
        }

        return '[unsupported]';
    }

    private function encodePayload(mixed $payload): string
    {
        $encoded = json_encode($payload, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES | JSON_PARTIAL_OUTPUT_ON_ERROR);

        if (!is_string($encoded)) {
            return '{}';
        }

        return $this->truncate($encoded);
    }

    private function truncate(string $value, int $maxLength = self::MAX_FIELD_LENGTH): string
    {
        if (function_exists('mb_strlen') && function_exists('mb_substr')) {
            return mb_strlen($value) > $maxLength
                ? mb_substr($value, 0, $maxLength - 14) . '[truncated]'
                : $value;
        }

        return strlen($value) > $maxLength
            ? substr($value, 0, $maxLength - 14) . '[truncated]'
            : $value;
    }

    private function isSensitiveKey(string $key): bool
    {
        return in_array(strtolower($key), self::SENSITIVE_KEYS, true);
    }

    private function fallbackToStderr(array $payload, Throwable $exception): void
    {
        $fallback = [
            'channel' => 'mysql',
            'level' => $payload['level'] ?? 'UNKNOWN',
            'title' => $payload['title'] ?? '',
            'uri' => $payload['uri'] ?? '',
            'error' => $exception->getMessage(),
        ];

        error_log('[mysql-logger-fallback] ' . json_encode($fallback, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES));
    }
}
