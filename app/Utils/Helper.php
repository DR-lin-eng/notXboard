<?php

namespace App\Utils;

use App\Services\Plugin\HookManager;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\Http;
use App\Models\User;

class Helper
{
    public static function uuidToBase64($uuid, $length)
    {
        return base64_encode(substr($uuid, 0, $length));
    }

    public static function getServerKey($timestamp, $length)
    {
        $key = self::getAppKeyBytes();
        $hash = hash_hmac('sha256', (string) $timestamp, $key, true);
        return base64_encode(substr($hash, 0, $length));
    }

    public static function guid($format = false)
    {
        $data = random_bytes(16);
        $data[6] = chr(ord($data[6]) & 0x0f | 0x40); // set version to 0100
        $data[8] = chr(ord($data[8]) & 0x3f | 0x80); // set bits 6-7 to 10
        $uuid = vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($data), 4));
        return $format ? $uuid : bin2hex($data);
    }

    public static function generateOrderNo(): string
    {
        $randomChar = mt_rand(10000, 99999);
        return date('YmdHms') . substr(microtime(), 2, 6) . $randomChar;
    }

    public static function exchange($from, $to)
    {
        $from = strtoupper(trim((string) $from));
        $to = strtoupper(trim((string) $to));
        if (!preg_match('/^[A-Z]{3,10}$/', $from) || !preg_match('/^[A-Z]{3,10}$/', $to)) {
            throw new \InvalidArgumentException('Invalid currency code');
        }

        $result = Http::timeout(10)
            ->connectTimeout(3)
            ->acceptJson()
            ->get('https://api.exchangerate.host/latest', [
                'symbols' => $to,
                'base' => $from,
            ])
            ->throw()
            ->json();

        return $result['rates'][$to] ?? null;
    }

    public static function randomChar($len, $special = false)
    {
        $chars = array(
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k",
            "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v",
            "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G",
            "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
            "S", "T", "U", "V", "W", "X", "Y", "Z", "0", "1", "2",
            "3", "4", "5", "6", "7", "8", "9"
        );

        if ($special) {
            $chars = array_merge($chars, array(
                "!", "@", "#", "$", "?", "|", "{", "/", ":", ";",
                "%", "^", "&", "*", "(", ")", "-", "_", "[", "]",
                "}", "<", ">", "~", "+", "=", ",", "."
            ));
        }

        $charsLen = count($chars) - 1;
        $str = '';
        for ($i = 0; $i < $len; $i++) {
            $str .= $chars[random_int(0, $charsLen)];
        }
        return $str;
    }

    public static function wrapIPv6($addr) {
        if (filter_var($addr, FILTER_VALIDATE_IP, FILTER_FLAG_IPV6)) {
            return "[$addr]";
        } else {
            return $addr;
        }
    }

    public static function multiPasswordVerify($algo, $salt, $password, $hash)
    {
        switch($algo) {
            case 'md5': return hash_equals((string) $hash, md5($password));
            case 'sha256': return hash_equals((string) $hash, hash('sha256', $password));
            case 'md5salt': return hash_equals((string) $hash, md5($password . $salt));
            default: return password_verify($password, $hash);
        }
    }

    public static function sanitizeForCsv(mixed $value): string
    {
        if ($value === null) {
            return '';
        }
        $stringValue = (string) $value;
        if ($stringValue === '') {
            return $stringValue;
        }
        $trimmed = ltrim($stringValue);
        if ($trimmed !== '' && in_array($trimmed[0], ['=', '+', '-', '@', "\t"], true)) {
            return "'" . $stringValue;
        }
        return $stringValue;
    }

    public static function emailSuffixVerify($email, $suffixs)
    {
        $suffix = preg_split('/@/', $email)[1];
        if (!$suffix) return false;
        if (!is_array($suffixs)) {
            $suffixs = preg_split('/,/', $suffixs);
        }
        if (!in_array($suffix, $suffixs)) return false;
        return true;
    }

    public static function trafficConvert(float $byte)
    {
        $kb = 1024;
        $mb = 1048576;
        $gb = 1073741824;
        if ($byte > $gb) {
            return round($byte / $gb, 2) . ' GB';
        } else if ($byte > $mb) {
            return round($byte / $mb, 2) . ' MB';
        } else if ($byte > $kb) {
            return round($byte / $kb, 2) . ' KB';
        } else if ($byte < 0) {
            return 0;
        } else {
            return round($byte, 2) . ' B';
        }
    }

    public static function getSubscribeUrl($userOrToken, $subscribeUrl = null): ?string
    {
        $user = null;
        if ($userOrToken instanceof User) {
            $user = $userOrToken;
        } elseif (is_array($userOrToken)) {
            $user = $userOrToken;
        } else {
            $token = (string) $userOrToken;
            if ($token !== '') {
                $user = User::where('token', $token)->first();
            }
        }

        if (!$user) {
            return null;
        }

        $token = null;
        $pathKey = null;
        $queryKey = null;
        $querySalt = null;

        if ($user instanceof User) {
            if (!$user->subscribe_path || !$user->subscribe_key || !$user->subscribe_salt) {
                $freshUser = $user->id ? User::where('id', $user->id)->first() : null;
                if ($freshUser) {
                    $user = $freshUser;
                }
            }
            $user->ensureSubscribeSecrets();
            $token = $user->token;
            $pathKey = $user->subscribe_path;
            $queryKey = $user->subscribe_key;
            $querySalt = $user->subscribe_salt;
        } elseif (is_array($user)) {
            $token = $user['token'] ?? null;
            $pathKey = $user['subscribe_path'] ?? null;
            $queryKey = $user['subscribe_key'] ?? null;
            $querySalt = $user['subscribe_salt'] ?? null;
            if (!$pathKey || !$queryKey || !$querySalt) {
                $model = $token ? User::where('token', $token)->first() : null;
                if (!$model) {
                    return null;
                }
                $model->ensureSubscribeSecrets();
                $token = $model->token;
                $pathKey = $model->subscribe_path;
                $queryKey = $model->subscribe_key;
                $querySalt = $model->subscribe_salt;
            }
        }

        if (!$token || !$pathKey || !$queryKey || !$querySalt) {
            return null;
        }

        $path = route('client.subscribe', ['path' => $pathKey], false);
        $queryString = http_build_query([
            $queryKey => $token,
            $querySalt => '1',
        ]);
        
        if ($subscribeUrl) {
            $finalUrl = rtrim($subscribeUrl, '/') . $path . '?' . $queryString;
            return HookManager::filter('subscribe.url', $finalUrl);
        }
        
        $subscribeUrlList = self::getSubscribeBaseUrls();

        if (empty($subscribeUrlList)) {
            return HookManager::filter('subscribe.url', url($path) . '?' . $queryString);
        }
        
        $selectedUrl = self::replaceByPattern(Arr::random($subscribeUrlList));
        $finalUrl = rtrim($selectedUrl, '/') . $path . '?' . $queryString;
        
        return HookManager::filter('subscribe.url', $finalUrl);
    }

    private static function getSubscribeBaseUrls(): array
    {
        $fullUrlList = collect(self::splitMultiValues((string) admin_setting('subscribe_url', '')))
            ->map(fn (string $item) => self::replaceByPattern($item))
            ->filter(fn ($item) => is_string($item) && trim($item) !== '')
            ->values()
            ->all();

        $rootDomainList = collect(self::splitMultiValues((string) admin_setting('subscribe_root_domains', '')))
            ->map(fn (string $item) => self::buildRandomSubscribeBaseUrl($item))
            ->filter(fn ($item) => is_string($item) && trim($item) !== '')
            ->values()
            ->all();

        return array_values(array_unique(array_merge($fullUrlList, $rootDomainList)));
    }

    private static function splitMultiValues(string $input): array
    {
        return array_values(array_filter(array_map(
            fn ($item) => trim((string) $item),
            preg_split('/[\s,]+/', $input) ?: []
        )));
    }

    private static function buildRandomSubscribeBaseUrl(string $raw): ?string
    {
        $value = trim($raw);
        if ($value === '') {
            return null;
        }

        if (str_contains($value, '://')) {
            $parsed = parse_url($value, PHP_URL_HOST);
            $value = is_string($parsed) ? $parsed : '';
        }

        $value = trim($value, " \t\n\r\0\x0B.");
        $value = preg_replace('/^\*\./', '', $value);
        if (!$value) {
            return null;
        }

        return 'https://' . self::randomLetters(6) . '.' . $value;
    }

    public static function randomPort($range): int {
        $portRange = explode('-', $range);
        return random_int((int)$portRange[0], (int)$portRange[1]);
    }

    public static function randomLetters(int $len): string
    {
        $chars = 'abcdefghijklmnopqrstuvwxyz';
        $max = strlen($chars) - 1;
        $result = '';
        for ($i = 0; $i < $len; $i++) {
            $result .= $chars[random_int(0, $max)];
        }
        return $result;
    }

    public static function base64EncodeUrlSafe($data)
    {
        $encoded = base64_encode($data);
        return str_replace(['+', '/', '='], ['-', '_', ''], $encoded);
    }

    /**
     * 根据规则替换域名中对应的字符串
     *
     * @param string $input 用户输入的字符串
     * @return string 替换后的字符串
     */
    public static function replaceByPattern($input)
    {
        $patterns = [
            '/\[(\d+)-(\d+)\]/' => function ($matches) {
                $min = intval($matches[1]);
                $max = intval($matches[2]);
                if ($min > $max) {
                    list($min, $max) = [$max, $min];
                }
                $randomNumber = rand($min, $max);
                return $randomNumber;
            },
            '/\[uuid\]/' => function () {
                return  self::guid(true);
            }
        ];
        foreach ($patterns as $pattern => $callback) {
            $input = preg_replace_callback($pattern, $callback, $input);
        }
        return $input;
    }

    public static function getIpByDomainName($domain) {
        return gethostbynamel($domain) ?: [];
    }

    public static function getRandFingerprint() {
        $fingerprints = ['chrome', 'firefox', 'safari', 'ios', 'edge', 'qq'];
        return Arr::random($fingerprints);
    }

    public static function encodeURIComponent($str) {
        $revert = array('%21'=>'!', '%2A'=>'*', '%27'=>"'", '%28'=>'(', '%29'=>')');
        return strtr(rawurlencode($str), $revert);
    }

    public static function getEmailSuffix(): array|bool
    {
        $suffix = admin_setting('email_whitelist_suffix', Dict::EMAIL_WHITELIST_SUFFIX_DEFAULT);
        if (!is_array($suffix)) {
            return preg_split('/,/', $suffix);
        }
        return $suffix;
    }
    
    /**
     * convert the transfer_enable to GB
     * @param float $transfer_enable
     * @return float
     */
    public static function transferToGB(float $transfer_enable): float
    {
        return $transfer_enable / 1073741824;
    }

    private static function getAppKeyBytes(): string
    {
        $key = (string) config('app.key', '');
        if (str_starts_with($key, 'base64:')) {
            $decoded = base64_decode(substr($key, 7), true);
            if ($decoded !== false) {
                return $decoded;
            }
        }
        return $key;
    }
}
