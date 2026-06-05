<?php

namespace App\Http\Controllers\V1\Guest;

use App\Http\Controllers\Controller;
use App\Models\Server;
use App\Models\StatServer;
use App\Models\User;
use App\Utils\Helper;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;

class PublicDashboardController extends Controller
{
    private const TRAFFIC_SNAPSHOT_KEY = 'public:traffic:snapshot';
    private const GEO_COUNTRIES_KEY = 'public:geo:countries';
    private const GEO_IP_PREFIX = 'public:geo:ip:';
    private const GEO_COUNTRY_PREFIX = 'public:geo:country:';
    private const GEO_TTL_SECONDS = 86400;

    public function overview(Request $request)
    {
        $this->recordCountryHit($request);

        $activeUsers = User::where('t', '>=', time() - 60)->count();
        $registeredUsers = User::count();
        $nodeCount = Server::count();

        $bandwidth = $this->getAverageBandwidth();

        return response()->json([
            'code' => 0,
            'message' => 'success',
            'data' => [
                'active_users' => $activeUsers,
                'registered_users' => $registeredUsers,
                'node_count' => $nodeCount,
                'avg_bandwidth' => $bandwidth,
            ],
        ]);
    }

    public function leaderboards(Request $request)
    {
        $this->recordCountryHit($request);

        $topUsers = User::select(['id', 'email', 'u', 'd'])
            ->orderByRaw('(u + d) DESC')
            ->limit(10)
            ->get()
            ->map(function (User $user) {
                $value = (int) (($user->u ?? 0) + ($user->d ?? 0));
                return [
                    'id' => (string) $user->id,
                    'name' => $this->maskIdentifier($user->email),
                    'value' => $value,
                    'value_text' => Helper::trafficConvert($value),
                ];
            })
            ->values();

        $nodeTotals = StatServer::where('record_type', 'd')
            ->selectRaw('server_id, SUM(u + d) as total')
            ->groupBy('server_id')
            ->orderBy('total', 'DESC')
            ->limit(10)
            ->get();

        $nodeNames = Server::whereIn('id', $nodeTotals->pluck('server_id'))
            ->get(['id', 'name'])
            ->pluck('name', 'id');

        $topNodes = $nodeTotals->map(function ($row) use ($nodeNames) {
            $value = (int) ($row->total ?? 0);
            $serverId = (int) $row->server_id;
            return [
                'id' => (string) $serverId,
                'name' => (string) ($nodeNames[$serverId] ?? ('Node ' . $serverId)),
                'value' => $value,
                'value_text' => Helper::trafficConvert($value),
            ];
        })->values();

        $regions = $this->getCountryCounts();

        return response()->json([
            'code' => 0,
            'message' => 'success',
            'data' => [
                'top_users' => $topUsers,
                'top_nodes' => $topNodes,
                'regions' => $regions,
            ],
        ]);
    }

    public function geo(Request $request)
    {
        $this->recordCountryHit($request);
        return response()->json([
            'code' => 0,
            'message' => 'success',
            'data' => [
                'regions' => $this->getCountryCounts(),
            ],
        ]);
    }

    private function getAverageBandwidth(): array
    {
        $now = time();
        $totalUpload = (int) User::sum('u');
        $totalDownload = (int) User::sum('d');

        $previous = Cache::get(self::TRAFFIC_SNAPSHOT_KEY);
        Cache::put(self::TRAFFIC_SNAPSHOT_KEY, [
            'ts' => $now,
            'u' => $totalUpload,
            'd' => $totalDownload,
        ], 120);

        $avgUp = 0;
        $avgDown = 0;

        if (is_array($previous) && isset($previous['ts'], $previous['u'], $previous['d'])) {
            $delta = $now - (int) $previous['ts'];
            if ($delta >= 20 && $delta <= 120) {
                $avgUp = max(0, (int) (($totalUpload - (int) $previous['u']) / $delta));
                $avgDown = max(0, (int) (($totalDownload - (int) $previous['d']) / $delta));
            }
        }

        return [
            'upload_bps' => $avgUp,
            'download_bps' => $avgDown,
        ];
    }

    private function maskIdentifier(?string $value): string
    {
        $value = (string) $value;
        if ($value === '') {
            return 'anonymous';
        }
        if (str_contains($value, '@')) {
            [$local, $domain] = explode('@', $value, 2);
            $localMasked = $this->maskText($local, 2, 1);
            $domainParts = explode('.', $domain);
            $domainHead = $domainParts[0] ?? $domain;
            $domainMasked = $this->maskText($domainHead, 1, 0);
            $domainTail = count($domainParts) > 1 ? '.' . implode('.', array_slice($domainParts, 1)) : '';
            return $localMasked . '@' . $domainMasked . $domainTail;
        }

        return $this->maskText($value, 2, 1);
    }

    private function maskText(string $value, int $keepStart, int $keepEnd): string
    {
        $len = strlen($value);
        if ($len <= $keepStart + $keepEnd) {
            return str_repeat('*', max(3, $len));
        }
        $start = substr($value, 0, $keepStart);
        $end = $keepEnd > 0 ? substr($value, -$keepEnd) : '';
        return $start . str_repeat('*', max(3, $len - $keepStart - $keepEnd)) . $end;
    }

    private function resolveCountry(Request $request): string
    {
        $headers = [
            'CF-IPCountry',
            'X-Country-Code',
            'X-Geo-Country',
            'X-Country',
        ];
        foreach ($headers as $header) {
            $value = strtoupper((string) $request->header($header, ''));
            if ($value !== '' && $value !== 'XX') {
                return $value;
            }
        }
        return 'ZZ';
    }

    private function recordCountryHit(Request $request): void
    {
        $ip = (string) $request->ip();
        if ($ip === '') {
            return;
        }
        $country = $this->resolveCountry($request);

        $ipKey = self::GEO_IP_PREFIX . $ip;
        if (Cache::has($ipKey)) {
            return;
        }

        Cache::put($ipKey, $country, self::GEO_TTL_SECONDS);

        $countryKey = self::GEO_COUNTRY_PREFIX . $country;
        if (Cache::has($countryKey)) {
            Cache::increment($countryKey);
        } else {
            Cache::put($countryKey, 1, self::GEO_TTL_SECONDS);
        }

        $countries = Cache::get(self::GEO_COUNTRIES_KEY, []);
        if (!in_array($country, $countries, true)) {
            $countries[] = $country;
            Cache::put(self::GEO_COUNTRIES_KEY, $countries, self::GEO_TTL_SECONDS);
        }
    }

    private function getCountryCounts(): array
    {
        $countries = Cache::get(self::GEO_COUNTRIES_KEY, []);
        $result = [];
        foreach ($countries as $country) {
            $count = (int) Cache::get(self::GEO_COUNTRY_PREFIX . $country, 0);
            if ($count > 0) {
                $result[] = [
                    'code' => $country,
                    'count' => $count,
                ];
            }
        }
        usort($result, fn($a, $b) => $b['count'] <=> $a['count']);
        return $result;
    }
}
