<?php

namespace App\Services;

use Illuminate\Support\Facades\Cache;

class TelegramMessageContextService
{
    private const CACHE_PREFIX = 'telegram:message-context:';
    private const DEFAULT_TTL_SECONDS = 604800;

    public function put(int $chatId, int $messageId, array $context, int $ttlSeconds = self::DEFAULT_TTL_SECONDS): void
    {
        Cache::put($this->cacheKey($chatId, $messageId), $context, max(60, $ttlSeconds));
    }

    public function get(int $chatId, int $messageId): ?array
    {
        $context = Cache::get($this->cacheKey($chatId, $messageId));

        return is_array($context) ? $context : null;
    }

    public function forget(int $chatId, int $messageId): void
    {
        Cache::forget($this->cacheKey($chatId, $messageId));
    }

    private function cacheKey(int $chatId, int $messageId): string
    {
        return self::CACHE_PREFIX . $chatId . ':' . $messageId;
    }
}
