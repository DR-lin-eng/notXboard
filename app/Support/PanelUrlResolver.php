<?php

namespace App\Support;

use Illuminate\Http\Request;
use InvalidArgumentException;

class PanelUrlResolver
{
    public static function fromRequest(Request $request): string
    {
        foreach (self::candidates($request) as $candidate) {
            $base = self::baseUrl($candidate);
            if ($base === null) {
                continue;
            }

            try {
                return UrlSecurity::normalizeHttpUrl($base);
            } catch (InvalidArgumentException) {
                continue;
            }
        }

        return 'http://localhost';
    }

    private static function candidates(Request $request): array
    {
        return [
            (string) (admin_setting('app_url', config('app.url')) ?: config('app.url')),
            (string) $request->getSchemeAndHttpHost(),
            (string) config('app.url'),
        ];
    }

    private static function baseUrl(string $value): ?string
    {
        $value = trim($value);
        if ($value === '') {
            return null;
        }

        $parts = parse_url($value);
        if (!isset($parts['scheme'], $parts['host'])) {
            return null;
        }

        $base = $parts['scheme'] . '://' . $parts['host'];
        if (isset($parts['port'])) {
            $base .= ':' . $parts['port'];
        }
        if (!empty($parts['path']) && $parts['path'] !== '/') {
            $base .= '/' . ltrim((string) $parts['path'], '/');
        }

        return $base;
    }
}
