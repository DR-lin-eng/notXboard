<?php

namespace App\Http\Controllers\V2\Admin;

use App\Http\Controllers\Controller;
use App\Models\Server;
use App\Models\StatServer;
use App\Models\User;
use App\Utils\Helper;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;

class AdminLeaderboardController extends Controller
{
    private const TRAFFIC_SNAPSHOT_KEY = 'admin:traffic:snapshot';
    private const GEO_COUNTRIES_KEY = 'public:geo:countries';
    private const GEO_COUNTRY_PREFIX = 'public:geo:country:';

    public function overview(Request $request)
    {
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
        $topUsers = User::select(['id', 'email', 'u', 'd'])
            ->orderByRaw('(u + d) DESC')
            ->limit(10)
            ->get()
            ->map(function (User $user) {
                $value = (int) (($user->u ?? 0) + ($user->d ?? 0));
                $email = (string) ($user->email ?? '');
                return [
                    'id' => (string) $user->id,
                    'name' => $email !== '' ? $email : ('User ' . $user->id),
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

        return response()->json([
            'code' => 0,
            'message' => 'success',
            'data' => [
                'top_users' => $topUsers,
                'top_nodes' => $topNodes,
                'regions' => $this->getCountryCounts(),
            ],
        ]);
    }

    public function geo(Request $request)
    {
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
