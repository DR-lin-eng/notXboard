<?php

namespace App\Services\LegacyTraffic;

use App\Models\User;

class LegacyTrafficUsageService
{
    public function applyTrafficFetch(array $server, array $data): void
    {
        $rate = (float) ($server['rate'] ?? 1);

        foreach ($data as $uid => $traffic) {
            User::where('id', (int) $uid)
                ->incrementEach(
                    [
                        'u' => (int) ($traffic[0] ?? 0) * $rate,
                        'd' => (int) ($traffic[1] ?? 0) * $rate,
                    ],
                    ['t' => time()]
                );
        }
    }
}
