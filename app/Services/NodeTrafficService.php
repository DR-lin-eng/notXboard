<?php

namespace App\Services;

use App\Models\NodeTrafficRecord;
use App\Models\ServerNode;
use App\Models\User;
use App\Models\UserTrafficUsageLog;
use Illuminate\Support\Facades\DB;

class NodeTrafficService
{
    /**
     * Record node traffic from a push payload.
     *
     * Supported payload formats:
     * 1. Legacy list: [[user_id, traffic], ...]
     * 2. V2bX map: { "user_id": [upload, download], ... }
     */
    public function recordTrafficFromPush(ServerNode $node, array $payload, ?array $allowedUserIds = null): void
    {
        $today = now()->toDateString();
        $quotaService = app(SubscriptionQuotaService::class);
        $normalizedTraffic = $this->normalizeTrafficPayload($payload);
        if ($allowedUserIds !== null) {
            $normalizedTraffic = $this->filterNormalizedTrafficByAllowedUserIds($normalizedTraffic, $allowedUserIds);
        }

        if (empty($normalizedTraffic)) {
            return;
        }

        DB::transaction(function () use ($node, $normalizedTraffic, $today, $quotaService) {
            $billedTrafficMap = $this->buildBilledTrafficMap($node, $normalizedTraffic);

            $this->upsertNodeTrafficRecords((int) $node->id, $today, $normalizedTraffic);
            $this->insertTrafficUsageLogs($node, $normalizedTraffic, $billedTrafficMap);
            $this->incrementUserTrafficCounters($normalizedTraffic);
            $quotaService->allocateBatchNodeTrafficToSubscriptions(
                (int) $node->id,
                $billedTrafficMap
            );

            $nodeTrafficDelta = collect($normalizedTraffic)->sum(function (array $traffic) {
                return max(0, (int) ($traffic['upload'] ?? 0)) + max(0, (int) ($traffic['download'] ?? 0));
            });
            if ($nodeTrafficDelta > 0) {
                $node->increment('traffic_used', $nodeTrafficDelta);
            }
        });
    }

    private function normalizeTrafficPayload(array $payload): array
    {
        $normalized = [];

        if (array_is_list($payload)) {
            foreach ($payload as $row) {
                if (!is_array($row) || count($row) !== 2) {
                    continue;
                }

                $userId = (int) ($row[0] ?? 0);
                $traffic = (int) ($row[1] ?? 0);
                if ($userId <= 0 || $traffic <= 0) {
                    continue;
                }

                $normalized[$userId] = [
                    'upload' => (int) (($normalized[$userId]['upload'] ?? 0)),
                    'download' => (int) (($normalized[$userId]['download'] ?? 0) + $traffic),
                ];
            }

            return $normalized;
        }

        foreach ($payload as $userId => $row) {
            if (!is_numeric($userId) || !is_array($row) || count($row) < 2) {
                continue;
            }

            $uid = (int) $userId;
            $upload = (int) ($row[0] ?? 0);
            $download = (int) ($row[1] ?? 0);
            if ($uid <= 0 || ($upload + $download) <= 0) {
                continue;
            }

            $normalized[$uid] = [
                'upload' => (int) (($normalized[$uid]['upload'] ?? 0) + $upload),
                'download' => (int) (($normalized[$uid]['download'] ?? 0) + $download),
            ];
        }

        return $normalized;
    }

    private function filterNormalizedTrafficByAllowedUserIds(array $normalizedTraffic, array $allowedUserIds): array
    {
        $allowedSet = array_fill_keys($this->normalizePositiveIntList($allowedUserIds), true);
        if (empty($allowedSet)) {
            return [];
        }

        return array_filter(
            $normalizedTraffic,
            fn ($traffic, $userId) => isset($allowedSet[(int) $userId]),
            ARRAY_FILTER_USE_BOTH
        );
    }

    private function normalizePositiveIntList(array $values): array
    {
        return collect($values)
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->unique()
            ->values()
            ->all();
    }

    public function getNodeTrafficSummary(ServerNode $node): array
    {
        $today = now()->toDateString();

        $todayRecord = NodeTrafficRecord::query()
            ->where('node_id', $node->id)
            ->where('record_date', $today)
            ->selectRaw('COALESCE(SUM(upload_traffic),0) as upload, COALESCE(SUM(download_traffic),0) as download')
            ->first();

        return [
            'node_id' => $node->id,
            'traffic_limit' => $node->traffic_limit,
            'traffic_used' => $node->traffic_used,
            'traffic_usage_percentage' => $node->getTrafficUsagePercentage(),
            'remaining_traffic' => $node->getRemainingTraffic(),
            'today' => [
                'upload' => (int) ($todayRecord?->upload ?? 0),
                'download' => (int) ($todayRecord?->download ?? 0),
            ],
        ];
    }

    private function buildBilledTrafficMap(ServerNode $node, array $normalizedTraffic): array
    {
        $result = [];
        $multiplier = $node->getEffectiveTrafficMultiplier();

        foreach ($normalizedTraffic as $userId => $traffic) {
            $upload = max(0, (int) ($traffic['upload'] ?? 0));
            $download = max(0, (int) ($traffic['download'] ?? 0));
            $total = $upload + $download;
            $billed = (int) ceil($total * $multiplier);

            if ((int) $userId > 0 && $billed > 0) {
                $result[(int) $userId] = $billed;
            }
        }

        return $result;
    }

    private function insertTrafficUsageLogs(ServerNode $node, array $normalizedTraffic, array $billedTrafficMap): void
    {
        $now = time();
        $nodeId = (int) $node->id;
        $multiplier = $node->getEffectiveTrafficMultiplier();
        $rows = [];

        foreach ($normalizedTraffic as $userId => $traffic) {
            $userId = (int) $userId;
            $raw = max(0, (int) ($traffic['upload'] ?? 0)) + max(0, (int) ($traffic['download'] ?? 0));
            $billed = max(0, (int) ($billedTrafficMap[$userId] ?? 0));

            if ($userId <= 0 || $raw <= 0) {
                continue;
            }

            $rows[] = [
                'user_id' => $userId,
                'node_id' => $nodeId,
                'raw_traffic_kb' => $raw,
                'billed_traffic_kb' => $billed,
                'multiplier_snapshot' => $multiplier,
                'source' => 'push',
                'recorded_at' => $now,
                'created_at' => $now,
                'updated_at' => $now,
            ];
        }

        if (!empty($rows)) {
            UserTrafficUsageLog::query()->insert($rows);
        }
    }

    private function upsertNodeTrafficRecords(int $nodeId, string $recordDate, array $normalizedTraffic): void
    {
        $rows = [];
        $now = now();

        foreach ($normalizedTraffic as $userId => $traffic) {
            $userId = (int) $userId;
            $upload = max(0, (int) ($traffic['upload'] ?? 0));
            $download = max(0, (int) ($traffic['download'] ?? 0));

            if ($userId <= 0 || ($upload + $download) <= 0) {
                continue;
            }

            $rows[] = [
                'user_id' => $userId,
                'node_id' => $nodeId,
                'record_date' => $recordDate,
                'upload_traffic' => $upload,
                'download_traffic' => $download,
                'created_at' => $now,
                'updated_at' => $now,
            ];
        }

        if (empty($rows)) {
            return;
        }

        $driver = config('database.default');
        if ($driver === 'sqlite') {
            foreach ($rows as $row) {
                $record = NodeTrafficRecord::query()->firstOrCreate(
                    [
                        'user_id' => $row['user_id'],
                        'node_id' => $row['node_id'],
                        'record_date' => $row['record_date'],
                    ],
                    [
                        'upload_traffic' => 0,
                        'download_traffic' => 0,
                    ]
                );

                if ((int) $row['upload_traffic'] > 0) {
                    $record->increment('upload_traffic', (int) $row['upload_traffic']);
                }
                if ((int) $row['download_traffic'] > 0) {
                    $record->increment('download_traffic', (int) $row['download_traffic']);
                }
            }

            return;
        }

        if ($driver === 'pgsql') {
            $this->upsertNodeTrafficRecordsForPostgres($rows);
            return;
        }

        NodeTrafficRecord::query()->upsert(
            $rows,
            ['user_id', 'node_id', 'record_date'],
            [
                'upload_traffic' => DB::raw('upload_traffic + VALUES(upload_traffic)'),
                'download_traffic' => DB::raw('download_traffic + VALUES(download_traffic)'),
                'updated_at' => $now,
            ]
        );
    }

    private function upsertNodeTrafficRecordsForPostgres(array $rows): void
    {
        $table = (new NodeTrafficRecord())->getTable();
        $placeholders = [];
        $bindings = [];

        foreach ($rows as $row) {
            $placeholders[] = '(?, ?, ?, ?, ?, ?, ?)';
            $bindings[] = (int) $row['user_id'];
            $bindings[] = (int) $row['node_id'];
            $bindings[] = (string) $row['record_date'];
            $bindings[] = (int) $row['upload_traffic'];
            $bindings[] = (int) $row['download_traffic'];
            $bindings[] = $row['created_at'];
            $bindings[] = $row['updated_at'];
        }

        $sql = "INSERT INTO {$table} (user_id, node_id, record_date, upload_traffic, download_traffic, created_at, updated_at)
            VALUES " . implode(', ', $placeholders) . "
            ON CONFLICT (user_id, node_id, record_date)
            DO UPDATE SET
                upload_traffic = {$table}.upload_traffic + EXCLUDED.upload_traffic,
                download_traffic = {$table}.download_traffic + EXCLUDED.download_traffic,
                updated_at = EXCLUDED.updated_at";

        DB::statement($sql, $bindings);
    }

    private function incrementUserTrafficCounters(array $normalizedTraffic): void
    {
        $deltas = [];

        foreach ($normalizedTraffic as $userId => $traffic) {
            $userId = (int) $userId;
            $upload = max(0, (int) ($traffic['upload'] ?? 0));
            $download = max(0, (int) ($traffic['download'] ?? 0));

            if ($userId <= 0 || ($upload + $download) <= 0) {
                continue;
            }

            $deltas[$userId] = [
                'upload' => $upload,
                'download' => $download,
            ];
        }

        if (empty($deltas)) {
            return;
        }

        $table = (new User())->getTable();
        $userIds = array_keys($deltas);
        $uploadCaseSegments = [];
        $downloadCaseSegments = [];
        $bindings = [];

        foreach ($deltas as $userId => $traffic) {
            $uploadCaseSegments[] = 'WHEN ? THEN ?';
            $bindings[] = (int) $userId;
            $bindings[] = (int) $traffic['upload'];
        }

        foreach ($deltas as $userId => $traffic) {
            $downloadCaseSegments[] = 'WHEN ? THEN ?';
            $bindings[] = (int) $userId;
            $bindings[] = (int) $traffic['download'];
        }

        $bindings[] = time();
        foreach ($userIds as $userId) {
            $bindings[] = (int) $userId;
        }

        $wherePlaceholders = implode(', ', array_fill(0, count($userIds), '?'));
        $sql = "UPDATE {$table}
            SET u = COALESCE(u, 0) + CASE id " . implode(' ', $uploadCaseSegments) . " ELSE 0 END,
                d = COALESCE(d, 0) + CASE id " . implode(' ', $downloadCaseSegments) . " ELSE 0 END,
                t = ?
            WHERE id IN ({$wherePlaceholders})";

        DB::update($sql, $bindings);
    }
}
