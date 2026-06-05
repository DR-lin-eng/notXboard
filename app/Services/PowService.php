<?php

namespace App\Services;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Redis;
use Illuminate\Support\Str;

class PowService
{
    private const CACHE_PREFIX = 'pow_challenge:';

    public function isEnabled(): bool
    {
        return (int) admin_setting('pow_enable', 0) === 1;
    }

    public function getConfiguredDifficulty(): int
    {
        $difficulty = (int) admin_setting('pow_difficulty', 4);
        return max(1, min(8, $difficulty));
    }

    public function getDifficulty(): int
    {
        $difficulty = $this->getConfiguredDifficulty();
        if (!$this->shouldAutoScale()) {
            return $difficulty;
        }

        $increment = $this->calculateDynamicIncrement(
            loadRatio: $this->getNormalizedLoadRatio(),
            queueBacklog: $this->getQueueBacklog()
        );

        return max(
            $difficulty,
            min($this->getAutoMaxDifficulty(), $difficulty + $increment)
        );
    }

    public function getTtl(): int
    {
        $ttl = (int) admin_setting('pow_ttl', 120);
        return max(30, min(600, $ttl));
    }

    public function getBaseValue(): string
    {
        return (string) admin_setting('pow_base_value', 'portal');
    }

    public function getSeedSalt(): string
    {
        return (string) admin_setting('pow_seed_salt', '');
    }

    public function requireJa3(): bool
    {
        return (int) admin_setting('pow_require_ja3', 1) === 1;
    }

    public function shouldAutoScale(): bool
    {
        return (int) admin_setting('pow_auto_scale_enable', 1) === 1;
    }

    public function getAutoMaxDifficulty(): int
    {
        $configured = (int) admin_setting('pow_auto_max_difficulty', 7);
        return max($this->getConfiguredDifficulty(), min(8, $configured));
    }

    public function generateChallenge(Request $request): array
    {
        if (!$this->isEnabled()) {
            return [false, [400, __('Pow disabled')]];
        }

        $ja3 = $this->getJa3($request);
        if ($this->requireJa3() && !$ja3) {
            return [false, [400, __('JA3 fingerprint missing')]];
        }

        $issuedAt = time();
        $ttl = $this->getTtl();
        $expiresAt = $issuedAt + $ttl;
        $difficulty = $this->getDifficulty();

        $seed = hash('sha256', random_bytes(32) . $this->getSeedSalt());
        $base = $this->getBaseValue();
        $challengeId = (string) Str::uuid();
        $token = $this->buildToken($ja3 ?? '', $issuedAt, $seed);

        Cache::put($this->getCacheKey($challengeId), [
            'seed' => $seed,
            'base' => $base,
            'difficulty' => $difficulty,
            'issued_at' => $issuedAt,
            'expires_at' => $expiresAt,
            'token' => $token,
            'ja3_hash' => $ja3 ? hash('sha256', $ja3) : null,
            'ip' => $request->ip(),
        ], $ttl);

        return [true, [
            'challenge_id' => $challengeId,
            'seed' => $seed,
            'base' => $base,
            'difficulty' => $difficulty,
            'issued_at' => $issuedAt,
            'expires_at' => $expiresAt,
            'token' => $token,
            'algo' => 'sha256-prefix-zeros',
        ]];
    }

    public function verify(Request $request): array
    {
        if (!$this->isEnabled()) {
            return [true, null];
        }

        $challengeId = (string) $request->input('pow_id');
        $nonce = (string) $request->input('pow_nonce');
        $clientToken = (string) $request->input('pow_token');

        if ($challengeId === '' || $nonce === '' || $clientToken === '') {
            return [false, [400, __('Pow data missing')]];
        }

        $challenge = Cache::pull($this->getCacheKey($challengeId));
        if (!$challenge) {
            return [false, [400, __('Pow challenge expired')]];
        }

        if (time() > (int) $challenge['expires_at']) {
            return [false, [400, __('Pow challenge expired')]];
        }

        $ja3 = $this->getJa3($request);
        if ($this->requireJa3() && !$ja3) {
            return [false, [400, __('JA3 fingerprint missing')]];
        }

        if ($ja3 && $challenge['ja3_hash'] && !hash_equals($challenge['ja3_hash'], hash('sha256', $ja3))) {
            return [false, [400, __('JA3 mismatch')]];
        }

        $expectedToken = $this->buildToken($ja3 ?? '', (int) $challenge['issued_at'], (string) $challenge['seed']);
        if (!hash_equals($expectedToken, $clientToken)) {
            return [false, [400, __('Pow token mismatch')]];
        }

        $input = implode('|', [
            (string) $challenge['seed'],
            (string) $challenge['base'],
            $expectedToken,
            $nonce
        ]);
        $hash = hash('sha256', $input);
        $prefix = str_repeat('0', (int) $challenge['difficulty']);
        if (strncmp($hash, $prefix, strlen($prefix)) !== 0) {
            return [false, [400, __('Pow verification failed')]];
        }

        return [true, null];
    }

    private function buildToken(string $ja3, int $issuedAt, string $seed): string
    {
        $secret = (string) config('app.key', 'pow');
        return hash_hmac('sha256', $ja3 . '|' . $issuedAt . '|' . $seed, $secret);
    }

    private function calculateDynamicIncrement(?float $loadRatio, ?int $queueBacklog): int
    {
        $increment = 0;

        if ($loadRatio !== null) {
            if ($loadRatio >= 1.5) {
                $increment = max($increment, 3);
            } elseif ($loadRatio >= 1.0) {
                $increment = max($increment, 2);
            } elseif ($loadRatio >= 0.7) {
                $increment = max($increment, 1);
            }
        }

        if ($queueBacklog !== null) {
            if ($queueBacklog >= 1000) {
                $increment = max($increment, 3);
            } elseif ($queueBacklog >= 300) {
                $increment = max($increment, 2);
            } elseif ($queueBacklog >= 100) {
                $increment = max($increment, 1);
            }
        }

        return $increment;
    }

    private function getNormalizedLoadRatio(): ?float
    {
        if (!function_exists('sys_getloadavg')) {
            return null;
        }

        $loads = sys_getloadavg();
        if (!is_array($loads) || !isset($loads[0]) || !is_numeric($loads[0])) {
            return null;
        }

        $cpuCount = $this->getCpuCoreCount();
        if ($cpuCount <= 0) {
            return null;
        }

        return ((float) $loads[0]) / $cpuCount;
    }

    private function getCpuCoreCount(): int
    {
        static $cpuCount = null;
        if ($cpuCount !== null) {
            return $cpuCount;
        }

        $cpuCount = 1;
        $cpuInfoPath = '/proc/cpuinfo';
        if (is_readable($cpuInfoPath)) {
            $content = (string) @file_get_contents($cpuInfoPath);
            $matches = [];
            preg_match_all('/^processor\s*:/m', $content, $matches);
            $count = count($matches[0] ?? []);
            if ($count > 0) {
                $cpuCount = $count;
            }
        }

        return $cpuCount;
    }

    private function getQueueBacklog(): ?int
    {
        try {
            $queueNames = [];

            $defaultQueue = (string) config('queue.connections.redis.queue', 'default');
            if ($defaultQueue !== '') {
                $queueNames[] = $defaultQueue;
            }

            $environments = config('horizon.environments', []);
            $appEnv = (string) config('app.env');
            $environmentConfig = is_array($environments[$appEnv] ?? null)
                ? $environments[$appEnv]
                : (is_array($environments['production'] ?? null) ? $environments['production'] : []);

            foreach ($environmentConfig as $supervisorConfig) {
                $queues = $supervisorConfig['queue'] ?? [];
                foreach ((array) $queues as $queueName) {
                    $queueName = trim((string) $queueName);
                    if ($queueName !== '') {
                        $queueNames[] = $queueName;
                    }
                }
            }

            $queueNames = array_values(array_unique(array_filter($queueNames)));
            if (empty($queueNames)) {
                return null;
            }

            $connection = (string) config('queue.connections.redis.connection', 'default');
            $redis = Redis::connection($connection);

            $backlog = 0;
            foreach ($queueNames as $queueName) {
                $backlog += (int) $redis->llen('queues:' . $queueName);
                $backlog += (int) $redis->zcard('queues:' . $queueName . ':delayed');
                $backlog += (int) $redis->zcard('queues:' . $queueName . ':reserved');
            }

            return $backlog;
        } catch (\Throwable) {
            return null;
        }
    }

    private function getCacheKey(string $challengeId): string
    {
        return self::CACHE_PREFIX . $challengeId;
    }

    private function getJa3(Request $request): ?string
    {
        $headers = [
            'x-ja3-fingerprint',
            'x-ja3',
            'x-ssl-ja3',
            'x-ja3-hash',
            'cf-ja3-hash',
        ];

        foreach ($headers as $header) {
            $value = $request->header($header);
            if (is_string($value) && trim($value) !== '') {
                return trim($value);
            }
        }

        return null;
    }
}
