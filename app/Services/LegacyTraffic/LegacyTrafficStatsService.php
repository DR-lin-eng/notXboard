<?php

namespace App\Services\LegacyTraffic;

use App\Models\StatServer;
use App\Models\StatUser;
use Illuminate\Support\Facades\DB;

class LegacyTrafficStatsService
{
    public function applyUserStat(array $server, array $data, string $recordType = 'd'): void
    {
        $recordAt = $this->resolveRecordAt($recordType);
        $rate = (float) ($server['rate'] ?? 1);

        foreach ($data as $uid => $traffic) {
            $this->upsertUserStat(
                (int) $uid,
                (int) ($traffic[0] ?? 0),
                (int) ($traffic[1] ?? 0),
                $rate,
                $recordAt,
                $recordType
            );
        }
    }

    public function applyServerStat(array $server, array $data, string $protocol, string $recordType = 'd'): void
    {
        $recordAt = $this->resolveRecordAt($recordType);
        $upload = 0;
        $download = 0;

        foreach ($data as $traffic) {
            $upload += (int) ($traffic[0] ?? 0);
            $download += (int) ($traffic[1] ?? 0);
        }

        $this->upsertServerStat(
            (int) ($server['id'] ?? 0),
            $protocol,
            $upload,
            $download,
            $recordAt,
            $recordType
        );
    }

    private function resolveRecordAt(string $recordType): int
    {
        return $recordType === 'm'
            ? strtotime(date('Y-m-01'))
            : strtotime(date('Y-m-d'));
    }

    private function upsertUserStat(
        int $uid,
        int $upload,
        int $download,
        float $rate,
        int $recordAt,
        string $recordType
    ): void {
        $driver = config('database.default');
        if ($driver === 'sqlite') {
            DB::transaction(function () use ($uid, $upload, $download, $rate, $recordAt, $recordType) {
                $existingRecord = StatUser::where([
                    'user_id' => $uid,
                    'server_rate' => $rate,
                    'record_at' => $recordAt,
                    'record_type' => $recordType,
                ])->first();

                if ($existingRecord) {
                    $existingRecord->update([
                        'u' => $existingRecord->u + ($upload * $rate),
                        'd' => $existingRecord->d + ($download * $rate),
                        'updated_at' => time(),
                    ]);
                } else {
                    StatUser::create([
                        'user_id' => $uid,
                        'server_rate' => $rate,
                        'record_at' => $recordAt,
                        'record_type' => $recordType,
                        'u' => ($upload * $rate),
                        'd' => ($download * $rate),
                        'created_at' => time(),
                        'updated_at' => time(),
                    ]);
                }
            }, 3);

            return;
        }

        if ($driver === 'pgsql') {
            $table = (new StatUser())->getTable();
            $now = time();
            $uploadAmount = ($upload * $rate);
            $downloadAmount = ($download * $rate);

            DB::statement(
                "INSERT INTO {$table} (user_id, server_rate, record_at, record_type, u, d, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT (user_id, server_rate, record_at)
                 DO UPDATE SET
                     u = {$table}.u + EXCLUDED.u,
                     d = {$table}.d + EXCLUDED.d,
                     updated_at = EXCLUDED.updated_at",
                [$uid, $rate, $recordAt, $recordType, $uploadAmount, $downloadAmount, $now, $now]
            );

            return;
        }

        StatUser::upsert(
            [
                'user_id' => $uid,
                'server_rate' => $rate,
                'record_at' => $recordAt,
                'record_type' => $recordType,
                'u' => ($upload * $rate),
                'd' => ($download * $rate),
                'created_at' => time(),
                'updated_at' => time(),
            ],
            ['user_id', 'server_rate', 'record_at', 'record_type'],
            [
                'u' => DB::raw("u + VALUES(u)"),
                'd' => DB::raw("d + VALUES(d)"),
                'updated_at' => time(),
            ]
        );
    }

    private function upsertServerStat(
        int $serverId,
        string $protocol,
        int $upload,
        int $download,
        int $recordAt,
        string $recordType
    ): void {
        $driver = config('database.default');
        if ($driver === 'sqlite') {
            DB::transaction(function () use ($serverId, $protocol, $upload, $download, $recordAt, $recordType) {
                $existingRecord = StatServer::where([
                    'record_at' => $recordAt,
                    'server_id' => $serverId,
                    'server_type' => $protocol,
                    'record_type' => $recordType,
                ])->first();

                if ($existingRecord) {
                    $existingRecord->update([
                        'u' => $existingRecord->u + $upload,
                        'd' => $existingRecord->d + $download,
                        'updated_at' => time(),
                    ]);
                } else {
                    StatServer::create([
                        'record_at' => $recordAt,
                        'server_id' => $serverId,
                        'server_type' => $protocol,
                        'record_type' => $recordType,
                        'u' => $upload,
                        'd' => $download,
                        'created_at' => time(),
                        'updated_at' => time(),
                    ]);
                }
            }, 3);

            return;
        }

        if ($driver === 'pgsql') {
            $table = (new StatServer())->getTable();
            $now = time();

            DB::statement(
                "INSERT INTO {$table} (record_at, server_id, server_type, record_type, u, d, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT (server_id, server_type, record_at)
                 DO UPDATE SET
                     u = {$table}.u + EXCLUDED.u,
                     d = {$table}.d + EXCLUDED.d,
                     updated_at = EXCLUDED.updated_at",
                [$recordAt, $serverId, $protocol, $recordType, $upload, $download, $now, $now]
            );

            return;
        }

        StatServer::upsert(
            [
                'record_at' => $recordAt,
                'server_id' => $serverId,
                'server_type' => $protocol,
                'record_type' => $recordType,
                'u' => $upload,
                'd' => $download,
                'created_at' => time(),
                'updated_at' => time(),
            ],
            ['server_id', 'server_type', 'record_at', 'record_type'],
            [
                'u' => DB::raw("u + VALUES(u)"),
                'd' => DB::raw("d + VALUES(d)"),
                'updated_at' => time(),
            ]
        );
    }
}
