<?php

namespace App\Http\Controllers\V1\Server;

use App\Http\Controllers\Controller;
use App\Jobs\UpdateAliveDataJob;
use App\Models\AuditLog;
use App\Models\ServerNode;
use App\Models\User;
use App\Models\UserOnlineSession;
use App\Services\AccessControlService;
use App\Services\AuditService;
use App\Services\LimitControlService;
use App\Services\NodeTrafficService;
use App\Services\CoreJobDispatchService;
use App\Services\LegacyTrafficDispatchService;
use App\Services\ServerService;
use App\Services\UserService;
use App\Utils\CacheKey;
use App\Utils\Helper;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;
use App\Services\UserOnlineService;
use Illuminate\Http\JsonResponse;

class UniProxyController extends Controller
{
    public function __construct(
        private readonly UserOnlineService $userOnlineService,
        private readonly AccessControlService $accessControlService,
        private readonly LimitControlService $limitControlService,
        private readonly NodeTrafficService $nodeTrafficService,
        private readonly AuditService $auditService,
    ) {
    }

    /**
     * 获取当前请求的节点信息
     */
    private function getNodeInfo(Request $request)
    {
        return $request->attributes->get('node_info');
    }

    private function isServerNode(mixed $node): bool
    {
        return $node instanceof ServerNode;
    }

    /**
     * Refresh ServerNode heartbeat timestamp from V2bX callbacks.
     */
    private function touchServerNodeHeartbeat(ServerNode $node): void
    {
        $cacheTime = max(300, (int) admin_setting('server_push_interval', 60) * 3);
        $nowTs = now()->timestamp;

        Cache::putMany([
            CacheKey::get('SERVER_SERVER_NODE_LAST_CHECK_AT', $node->id) => $nowTs,
            CacheKey::get('SERVER_SERVER_NODE_LAST_LOAD_AT', $node->id) => $nowTs,
        ], $cacheTime);
    }

    private function getServerNodeUserSnapshot(ServerNode $node, mixed $credentialService): array
    {
        $limit = $this->resolveServerNodeUserResponseLimit();
        $cacheTtl = $this->resolveServerNodeUserSnapshotTtl();
        $cacheKey = sprintf('server_node:user_snapshot:%d:%d', (int) $node->id, $limit);

        return Cache::remember(
            $cacheKey,
            $cacheTtl,
            fn () => $this->buildServerNodeUserSnapshot($node, $credentialService, $limit)
        );
    }

    private function buildServerNodeUserSnapshot(ServerNode $node, mixed $credentialService, int $limit): array
    {
        $queryLimit = max(1, $limit + 1);
        $accessibleUserIds = $this->accessControlService->getAccessibleUserIdsForNode($node, $queryLimit);
        $isTruncated = count($accessibleUserIds) > $limit;
        if ($isTruncated) {
            $accessibleUserIds = array_slice($accessibleUserIds, 0, $limit);
        }

        if (empty($accessibleUserIds)) {
            return [];
        }

        $responseUsers = [];
        foreach (array_chunk($accessibleUserIds, 1000) as $userIdChunk) {
            $users = User::query()
                ->select([
                    'id',
                    'uuid',
                    'subscription_credential_version',
                    'trust_level',
                    'is_silenced',
                    'expired_at',
                    'is_super_admin',
                ])
                ->with('individualLimit')
                ->whereIn('id', $userIdChunk)
                ->orderBy('id')
                ->get();

            $effectiveLimitsMap = $this->limitControlService->getEffectiveLimitsForUsers($users);
            foreach ($users as $user) {
                $effectiveLimits = $effectiveLimitsMap[(int) $user->id] ?? [];
                $responseUsers[] = $this->buildServerNodeUserPayloadRow($user, $node, $effectiveLimits, $credentialService);
            }
        }

        if ($isTruncated) {
            Log::warning('UniProxy user response truncated by configured limit', [
                'node_id' => (int) $node->id,
                'node_v2bx_node_id' => (int) ($node->v2bx_node_id ?? 0),
                'configured_limit' => $limit,
                'accessible_user_count_overflow' => true,
            ]);
        }

        return $responseUsers;
    }

    private function buildServerNodeUserPayloadRow(
        User $user,
        ServerNode $node,
        array $effectiveLimits,
        mixed $credentialService
    ): array {
        $deviceLimit = (int) ($effectiveLimits['device_limit'] ?? 0);
        if ($node->device_limit > 0 && ($deviceLimit <= 0 || $deviceLimit > $node->device_limit)) {
            $deviceLimit = (int) $node->device_limit;
        }

        $connectionLimit = (int) ($effectiveLimits['connection_limit'] ?? 0);
        if ($node->connection_limit > 0 && ($connectionLimit <= 0 || $connectionLimit > $node->connection_limit)) {
            $connectionLimit = (int) $node->connection_limit;
        }

        $speedLimitDown = (int) ($effectiveLimits['speed_limit_down'] ?? 0);
        if ($node->speed_limit_down > 0 && ($speedLimitDown <= 0 || $speedLimitDown > $node->speed_limit_down)) {
            $speedLimitDown = (int) $node->speed_limit_down;
        }

        return [
            'id' => (int) $user->id,
            'uuid' => (string) $credentialService->getEffectiveUuid($user),
            'speed_limit' => $speedLimitDown,
            'device_limit' => $deviceLimit,
            'connection_limit' => $connectionLimit,
            'trust_level' => (int) ($user->trust_level ?? 0),
            'is_silenced' => (bool) ($user->is_silenced ?? false),
        ];
    }

    private function resolveServerNodeUserResponseLimit(): int
    {
        $limit = (int) admin_setting(
            'server_node_user_response_limit',
            env('SERVER_NODE_USER_RESPONSE_LIMIT', 20000)
        );

        if ($limit <= 0) {
            return 20000;
        }

        return min($limit, 100000);
    }

    private function resolveServerNodeUserSnapshotTtl(): int
    {
        $ttl = (int) admin_setting('server_node_user_snapshot_ttl', 15);
        if ($ttl <= 0) {
            return 15;
        }

        return min(max($ttl, 5), 120);
    }

    private function extractCandidateUserIdsFromPushPayload(array $payload): array
    {
        $userIds = [];
        if (array_is_list($payload)) {
            foreach ($payload as $row) {
                if (!is_array($row) || count($row) < 1 || !is_numeric($row[0] ?? null)) {
                    continue;
                }
                $userIds[] = (int) $row[0];
            }

            return $this->normalizePositiveIntList($userIds);
        }

        foreach ($payload as $userId => $row) {
            if (!is_numeric($userId)) {
                continue;
            }
            $userIds[] = (int) $userId;
        }

        return $this->normalizePositiveIntList($userIds);
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

    private function logDroppedNodeUsers(
        string $source,
        ServerNode $node,
        array $droppedUserIds,
        int $candidateUserCount
    ): void {
        $droppedUserIds = $this->normalizePositiveIntList($droppedUserIds);
        if (empty($droppedUserIds)) {
            return;
        }

        Log::warning('UniProxy dropped unauthorized node users', [
            'source' => $source,
            'node_id' => (int) $node->id,
            'node_v2bx_node_id' => (int) ($node->v2bx_node_id ?? 0),
            'candidate_user_count' => max(0, (int) $candidateUserCount),
            'dropped_user_count' => count($droppedUserIds),
            'sample_user_ids' => $this->sampleUserIds($droppedUserIds),
        ]);
    }

    private function sampleUserIds(array $userIds, int $sampleSize = 20): array
    {
        return array_values(array_slice($this->normalizePositiveIntList($userIds), 0, max(1, $sampleSize)));
    }

    // 后端获取用户
    public function user(Request $request)
    {
        $node = $this->getNodeInfo($request);
        $credentialService = app(\App\Services\SubscriptionCredentialService::class);
        if ($this->isServerNode($node)) {
            $this->touchServerNodeHeartbeat($node);
            if (!$node->isActive() || $node->isTrafficExceeded()) {
                $response['users'] = [];
            } else {
                $response['users'] = $this->getServerNodeUserSnapshot($node, $credentialService);
            }
        } else {
            $nodeType = $node->type;
            $nodeId = $node->id;
            Cache::put(CacheKey::get('SERVER_' . strtoupper($nodeType) . '_LAST_CHECK_AT', $nodeId), time(), 3600);
            $users = ServerService::getAvailableUsers($node);
            $response['users'] = $users;
        }

        $eTag = sha1(json_encode($response));
        if (strpos($request->header('If-None-Match', ''), $eTag) !== false) {
            return response(null, 304);
        }

        return response($response)->header('ETag', "\"{$eTag}\"");
    }

    // 后端提交数据
    public function push(Request $request)
    {
        $res = json_decode(request()->getContent(), true);
        if (!is_array($res)) {
            return $this->fail([422, 'Invalid data format']);
        }
        $node = $this->getNodeInfo($request);

        if ($this->isServerNode($node)) {
            $this->touchServerNodeHeartbeat($node);
            $candidateUserIds = $this->extractCandidateUserIdsFromPushPayload($res);
            $allowedUserIds = $this->accessControlService->filterAccessibleUserIdsForNode($node, $candidateUserIds);
            $droppedUserIds = array_values(array_diff($candidateUserIds, $allowedUserIds));
            if (!empty($droppedUserIds)) {
                $this->logDroppedNodeUsers('push', $node, $droppedUserIds, count($candidateUserIds));
            }

            $this->nodeTrafficService->recordTrafficFromPush($node, $res, $allowedUserIds);
            return $this->success(true);
        }

        $data = array_filter($res, function ($item) {
            return is_array($item)
                && count($item) === 2
                && is_numeric($item[0])
                && is_numeric($item[1]);
        });
        if (empty($data)) {
            return $this->success(true);
        }

        $nodeType = $node->type;
        $nodeId = $node->id;

        Cache::put(
            CacheKey::get('SERVER_' . strtoupper($nodeType) . '_ONLINE_USER', $nodeId),
            count($data),
            3600
        );
        Cache::put(
            CacheKey::get('SERVER_' . strtoupper($nodeType) . '_LAST_PUSH_AT', $nodeId),
            time(),
            3600
        );

        $userService = new UserService();
        $userService->trafficFetch($node, $nodeType, $data);
        return $this->success(true);
    }

    // 后端获取配置
    public function config(Request $request)
    {
        $node = $this->getNodeInfo($request);
        if ($this->isServerNode($node)) {
            $this->touchServerNodeHeartbeat($node);
            $response = $node->getV2bXPanelConfig();
            $response['base_config'] = [
                'push_interval' => (int) admin_setting('server_push_interval', 60),
                'pull_interval' => (int) admin_setting('server_pull_interval', 60),
            ];

            $eTag = sha1(json_encode($response));
            if (strpos($request->header('If-None-Match', ''), $eTag) !== false) {
                return response(null, 304);
            }
            return response($response)->header('ETag', "\"{$eTag}\"");
        }

        $nodeType = $node->type;
        $protocolSettings = $node->protocol_settings;

        $serverPort = $node->server_port;
        $host = $node->host;

        $baseConfig = [
            'protocol' => $nodeType,
            'listen_ip' => '0.0.0.0',
            'server_port' => (int) $serverPort,
            'network' => data_get($protocolSettings, 'network'),
            'networkSettings' => data_get($protocolSettings, 'network_settings') ?: null,
        ];

        $response = match ($nodeType) {
            'shadowsocks' => [
                ...$baseConfig,
                'cipher' => $protocolSettings['cipher'],
                'plugin' => $protocolSettings['plugin'],
                'plugin_opts' => $protocolSettings['plugin_opts'],
                'server_key' => match ($protocolSettings['cipher']) {
                        '2022-blake3-aes-128-gcm' => Helper::getServerKey($node->created_at, 16),
                        '2022-blake3-aes-256-gcm' => Helper::getServerKey($node->created_at, 32),
                        default => null
                    }
            ],
            'vmess' => [
                ...$baseConfig,
                'tls' => (int) $protocolSettings['tls']
            ],
            'trojan' => [
                ...$baseConfig,
                'host' => $host,
                'server_name' => $protocolSettings['server_name'],
            ],
            'vless' => [
                ...$baseConfig,
                'tls' => (int) $protocolSettings['tls'],
                'flow' => $protocolSettings['flow'],
                'tls_settings' =>
                        match ((int) $protocolSettings['tls']) {
                            2 => $protocolSettings['reality_settings'],
                            default => $protocolSettings['tls_settings']
                        }
            ],
            'hysteria' => [
                ...$baseConfig,
                'server_port' => (int) $serverPort,
                'version' => (int) $protocolSettings['version'],
                'host' => $host,
                'server_name' => $protocolSettings['tls']['server_name'],
                'up_mbps' => (int) $protocolSettings['bandwidth']['up'],
                'down_mbps' => (int) $protocolSettings['bandwidth']['down'],
                ...match ((int) $protocolSettings['version']) {
                        1 => ['obfs' => $protocolSettings['obfs']['password'] ?? null],
                        2 => [
                            'obfs' => $protocolSettings['obfs']['open'] ? $protocolSettings['obfs']['type'] : null,
                            'obfs-password' => $protocolSettings['obfs']['password'] ?? null
                        ],
                        default => []
                    }
            ],
            'tuic' => [
                ...$baseConfig,
                'version' => (int) $protocolSettings['version'],
                'server_port' => (int) $serverPort,
                'server_name' => $protocolSettings['tls']['server_name'],
                'congestion_control' => $protocolSettings['congestion_control'],
                'auth_timeout' => '3s',
                'zero_rtt_handshake' => (bool) ($protocolSettings['zero_rtt_handshake'] ?? false),
                'heartbeat' => (string) ($protocolSettings['heartbeat'] ?? '10s'),
            ],
            'anytls' => [
                ...$baseConfig,
                'server_port' => (int) $serverPort,
                'server_name' => $protocolSettings['tls']['server_name'],
                'padding_scheme' => $protocolSettings['padding_scheme'],
            ],
            'socks' => [
                ...$baseConfig,
                'server_port' => (int) $serverPort,
            ],
            'naive' => [
                ...$baseConfig,
                'server_port' => (int) $serverPort,
                'tls' => (int) $protocolSettings['tls'],
                'tls_settings' => $protocolSettings['tls_settings']
            ],
            'http' => [
                ...$baseConfig,
                'server_port' => (int) $serverPort,
                'tls' => (int) $protocolSettings['tls'],
                'tls_settings' => $protocolSettings['tls_settings']
            ],
            'mieru' => [
                ...$baseConfig,
                'server_port' => (string) $serverPort,
                'protocol' => (int) $protocolSettings['protocol'],
            ],
            default => []
        };

        $response['base_config'] = [
            'push_interval' => (int) admin_setting('server_push_interval', 60),
            'pull_interval' => (int) admin_setting('server_pull_interval', 60)
        ];

        if (!empty($node['route_ids'])) {
            $response['routes'] = ServerService::getRoutes($node['route_ids']);
        }

        $eTag = sha1(json_encode($response));
        if (strpos($request->header('If-None-Match', ''), $eTag) !== false) {
            return response(null, 304);
        }
        return response($response)->header('ETag', "\"{$eTag}\"");
    }

    // 获取在线用户数据（wyx2685
    public function alivelist(Request $request): JsonResponse
    {
        $node = $this->getNodeInfo($request);

        if ($this->isServerNode($node)) {
            $users = $this->accessControlService->getAccessibleUsersForNode($node);
            $effectiveLimitsMap = $this->limitControlService->getEffectiveLimitsForUsers($users);
            $deviceLimitUsers = $users->filter(function ($user) use ($node, $effectiveLimitsMap) {
                $deviceLimit = (int) ($effectiveLimitsMap[(int) $user->id]['device_limit'] ?? 0);
                if ($node->device_limit > 0 && ($deviceLimit <= 0 || $deviceLimit > $node->device_limit)) {
                    $deviceLimit = (int) $node->device_limit;
                }

                return $deviceLimit > 0;
            })->values();
        } else {
            $deviceLimitUsers = ServerService::getAvailableUsers($node)
                ->where('device_limit', '>', 0);
        }

        $alive = $this->userOnlineService->getAliveList($deviceLimitUsers);
        return response()->json(['alive' => (object) $alive]);
    }

    // 后端提交在线数据
    public function alive(Request $request): JsonResponse
    {
        $node = $this->getNodeInfo($request);
        $data = json_decode(request()->getContent(), true);
        if ($data === null) {
            return response()->json([
                'error' => 'Invalid online data'
            ], 400);
        }
        if ($this->isServerNode($node)) {
            $this->touchServerNodeHeartbeat($node);
            $now = now();
            $candidateUserIds = $this->normalizePositiveIntList(array_keys($data));
            $allowedUserIds = $this->accessControlService->filterAccessibleUserIdsForNode($node, $candidateUserIds);
            $allowedUserSet = array_fill_keys($allowedUserIds, true);
            $droppedUserIds = [];

            foreach ($data as $userId => $ips) {
                $userId = (int) $userId;
                if ($userId <= 0 || !is_array($ips)) {
                    continue;
                }
                if (!isset($allowedUserSet[$userId])) {
                    $droppedUserIds[$userId] = true;
                    continue;
                }

                $ipList = collect($ips)
                    ->filter(fn ($ip) => is_string($ip) && $ip !== '')
                    ->unique()
                    ->values()
                    ->all();

                if (empty($ipList)) {
                    continue;
                }

                UserOnlineSession::query()
                    ->where('user_id', $userId)
                    ->where('node_id', $node->id)
                    ->whereNotIn('ip_address', $ipList)
                    ->delete();

                foreach ($ipList as $ip) {
                    UserOnlineSession::query()->updateOrCreate(
                        [
                            'user_id' => $userId,
                            'node_id' => $node->id,
                            'ip_address' => $ip,
                        ],
                        [
                            'connection_count' => 1,
                            'last_activity' => $now,
                        ]
                    );
                }
            }

            if (!empty($droppedUserIds)) {
                $this->logDroppedNodeUsers(
                    'alive',
                    $node,
                    array_map('intval', array_keys($droppedUserIds)),
                    count($candidateUserIds)
                );
            }

            return response()->json(['data' => true]);
        }

        $dispatchService = app(CoreJobDispatchService::class);
        if ($dispatchService->shouldDispatchSync()) {
            app(LegacyTrafficDispatchService::class)->updateAliveData($data, $node->type, $node->id);
        } else {
            $dispatchService->dispatch(new UpdateAliveDataJob($data, $node->type, $node->id));
        }
        return response()->json(['data' => true]);
    }

    // 提交节点负载状态
    public function status(Request $request): JsonResponse
    {
        $node = $this->getNodeInfo($request);
        if ($this->isServerNode($node)) {
            $this->touchServerNodeHeartbeat($node);
        }

        $data = $request->validate([
            'cpu' => 'required|numeric|min:0|max:100',
            'mem.total' => 'required|integer|min:0',
            'mem.used' => 'required|integer|min:0',
            'swap.total' => 'required|integer|min:0',
            'swap.used' => 'required|integer|min:0',
            'disk.total' => 'required|integer|min:0',
            'disk.used' => 'required|integer|min:0',
        ]);

        $nodeType = $this->isServerNode($node) ? 'server_node' : $node->type;
        $nodeId = $node->id;

        $statusData = [
            'cpu' => (float) $data['cpu'],
            'mem' => [
                'total' => (int) $data['mem']['total'],
                'used' => (int) $data['mem']['used'],
            ],
            'swap' => [
                'total' => (int) $data['swap']['total'],
                'used' => (int) $data['swap']['used'],
            ],
            'disk' => [
                'total' => (int) $data['disk']['total'],
                'used' => (int) $data['disk']['used'],
            ],
            'updated_at' => now()->timestamp,
        ];

        $cacheTime = max(300, (int) admin_setting('server_push_interval', 60) * 3);
        Cache::putMany([
            CacheKey::get('SERVER_' . strtoupper($nodeType) . '_LOAD_STATUS', $nodeId) => $statusData,
            CacheKey::get('SERVER_' . strtoupper($nodeType) . '_LAST_LOAD_AT', $nodeId) => now()->timestamp,
        ], $cacheTime);

        return response()->json(['data' => true, "code" => 0, "message" => "success"]);
    }

    /**
     * Node-side audit event reporting (ServerNode only).
     */
    public function audit(Request $request): JsonResponse
    {
        $node = $this->getNodeInfo($request);
        if (!$this->isServerNode($node)) {
            return response()->json(['success' => false, 'error' => 'Audit only supported for ServerNode'], 400);
        }

        $data = $request->validate([
            'user_id' => 'required|integer|min:1',
            'ip_address' => 'required|string|max:45',
            'target_domain' => 'nullable|string|max:255',
            'target_protocol' => 'nullable|string|max:50',
            'action_taken' => 'nullable|in:' . implode(',', [
                AuditLog::ACTION_ALLOWED,
                AuditLog::ACTION_BLOCKED,
                AuditLog::ACTION_LOGGED,
            ]),
        ]);

        $log = null;
        if (!empty($data['action_taken'])) {
            $log = AuditLog::createLog(
                userId: (int) $data['user_id'],
                nodeId: (int) $node->id,
                ipAddress: (string) $data['ip_address'],
                actionTaken: (string) $data['action_taken'],
                ruleId: null,
                targetDomain: $data['target_domain'] ?? null,
                targetProtocol: $data['target_protocol'] ?? null
            );
        } else {
            $log = $this->auditService->evaluateAndLog(
                node: $node,
                userId: (int) $data['user_id'],
                ipAddress: (string) $data['ip_address'],
                targetDomain: $data['target_domain'] ?? null,
                targetProtocol: $data['target_protocol'] ?? null
            );
        }

        return response()->json([
            'success' => true,
            'data' => [
                'logged' => $log !== null,
                'log_id' => $log?->id,
            ],
        ]);
    }
}
