<?php

namespace App\Services\LegacyTraffic;

use App\Models\User;
use App\Services\UserOnlineService;
use Illuminate\Support\Facades\Cache;

class LegacyOnlineStatusService
{
    public function updateAliveData(array $data, string $nodeType, int $nodeId): void
    {
        $updateAt = time();
        $now = now();
        $nodeKey = $nodeType . $nodeId;
        $userUpdates = $this->buildUserUpdates($data, $nodeKey, $updateAt);

        if (empty($userUpdates)) {
            return;
        }

        $existingUsers = $this->resolveExistingUserSet($userUpdates);
        if (empty($existingUsers)) {
            return;
        }

        $this->persistOnlineCounts($userUpdates, $existingUsers, $now);
    }

    private function buildUserUpdates(array $data, string $nodeKey, int $updateAt): array
    {
        $userUpdates = [];

        foreach ($data as $uid => $ips) {
            $cacheKey = 'ALIVE_IP_USER_' . $uid;
            $ipsArray = Cache::get($cacheKey, []);
            $ipsArray = [
                ...collect($ipsArray)->filter(
                    fn (mixed $value): bool => is_array($value) && ($updateAt - ($value['lastupdateAt'] ?? 0) <= 100)
                ),
                $nodeKey => [
                    'aliveips' => $ips,
                    'lastupdateAt' => $updateAt,
                ],
            ];

            $count = UserOnlineService::calculateDeviceCount($ipsArray);
            $ipsArray['alive_ip'] = $count;
            Cache::put($cacheKey, $ipsArray, now()->addSeconds(120));

            $userUpdates[] = [
                'id' => (int) $uid,
                'count' => (int) $count,
            ];
        }

        return $userUpdates;
    }

    private function resolveExistingUserSet(array $userUpdates): array
    {
        $allIds = collect($userUpdates)
            ->pluck('id')
            ->filter()
            ->map(fn ($value) => (int) $value)
            ->unique()
            ->values()
            ->all();

        if (empty($allIds)) {
            return [];
        }

        $existingIds = User::query()
            ->whereIn('id', $allIds)
            ->pluck('id')
            ->map(fn ($value) => (int) $value)
            ->all();

        return array_fill_keys($existingIds, true);
    }

    private function persistOnlineCounts(array $userUpdates, array $existingUsers, mixed $now): void
    {
        collect($userUpdates)
            ->filter(fn ($row) => isset($existingUsers[(int) ($row['id'] ?? 0)]))
            ->chunk(1000)
            ->each(function ($chunk) use ($now) {
                collect($chunk)->each(function ($update) use ($now) {
                    $id = (int) ($update['id'] ?? 0);
                    if ($id <= 0) {
                        return;
                    }

                    User::query()
                        ->whereKey($id)
                        ->update([
                            'online_count' => (int) ($update['count'] ?? 0),
                            'last_online_at' => $now,
                        ]);
                });
            });
    }
}
