<?php

namespace App\Support;

use InvalidArgumentException;

class UrlSecurity
{
    public static function normalizeHttpUrl(string $url, bool $blockPrivateHosts = false): string
    {
        $url = trim($url);
        if ($url === '' || preg_match('/[\x00-\x1F\x7F]/', $url)) {
            throw new InvalidArgumentException('Invalid URL');
        }

        $parts = parse_url($url);
        $scheme = strtolower((string) ($parts['scheme'] ?? ''));
        $host = (string) ($parts['host'] ?? '');
        if (!in_array($scheme, ['http', 'https'], true) || $host === '') {
            throw new InvalidArgumentException('URL must use http or https and include a host');
        }
        if (isset($parts['user']) || isset($parts['pass'])) {
            throw new InvalidArgumentException('URL userinfo is not allowed');
        }
        if ($blockPrivateHosts && self::isPrivateOrReservedHost($host)) {
            throw new InvalidArgumentException('Private or reserved hosts are not allowed');
        }

        return rtrim($url, '/');
    }

    public static function normalizeRelativePath(?string $path, string $default): string
    {
        $path = trim((string) ($path ?: $default));
        if ($path === '' || preg_match('/[\x00-\x1F\x7F]/', $path)) {
            throw new InvalidArgumentException('Invalid path');
        }
        if (preg_match('/^[a-z][a-z0-9+.-]*:/i', $path) || str_starts_with($path, '//')) {
            throw new InvalidArgumentException('Path must be relative to the payment gateway host');
        }
        if (str_contains($path, '?') || str_contains($path, '#') || str_contains($path, '\\')) {
            throw new InvalidArgumentException('Path must not contain query strings, fragments, or backslashes');
        }

        return '/' . ltrim($path, '/');
    }

    private static function isPrivateOrReservedHost(string $host): bool
    {
        $normalized = trim($host, "[] \t\n\r\0\x0B.");
        if ($normalized === '' || strcasecmp($normalized, 'localhost') === 0 || str_ends_with(strtolower($normalized), '.localhost')) {
            return true;
        }

        if (filter_var($normalized, FILTER_VALIDATE_IP)) {
            return filter_var(
                $normalized,
                FILTER_VALIDATE_IP,
                FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE
            ) === false;
        }

        return false;
    }
}
