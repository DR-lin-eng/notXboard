<?php

namespace App\Services;

use App\Models\Server;
use App\Models\ServerNode;
use App\Models\ServerRoute;
use App\Models\User;
use App\Services\Plugin\HookManager;
use App\Utils\Helper;
use Carbon\CarbonInterface;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Log;
use Throwable;

class ServerService
{

    /**
     * 获取所有服务器列表
     * @return Collection
     */
    public static function getAllServers(): Collection
    {
        $query = Server::orderBy('sort', 'ASC');

        return $query->get()->append([
            'last_check_at',
            'last_push_at',
            'online',
            'is_online',
            'available_status',
            'cache_key',
            'load_status'
        ]);
    }

    /**
     * 获取指定用户可用的服务器列表
     * @param User $user
     * @return array
     */
    public static function getAvailableServers(User $user): array
    {
        $legacyServers = self::getLegacyAvailableServers($user);
        $serverNodeServers = self::getServerNodeAvailableServers($user);

        if (empty($serverNodeServers)) {
            return $legacyServers;
        }
        if (empty($legacyServers)) {
            return $serverNodeServers;
        }

        $merged = [];
        foreach (array_merge($legacyServers, $serverNodeServers) as $server) {
            $cacheKey = (string) ($server['cache_key'] ?? '');
            if ($cacheKey === '') {
                $cacheKey = sprintf(
                    'server:%s:%s',
                    (string) ($server['type'] ?? 'unknown'),
                    (string) ($server['id'] ?? uniqid('', true))
                );
            }
            $merged[$cacheKey] = $server;
        }

        return array_values($merged);
    }

    /**
     * Legacy v2_server nodes available for user.
     */
    private static function getLegacyAvailableServers(User $user): array
    {
        $servers = Server::whereJsonContains('group_ids', (string) $user->group_id)
            ->where('show', true)
            ->orderBy('sort', 'ASC')
            ->get()
            ->append(['last_check_at', 'last_push_at', 'online', 'is_online', 'available_status', 'cache_key', 'server_key'])
            ->filter(fn (Server $server) => (int) $server->available_status !== Server::STATUS_OFFLINE);

        $servers = collect($servers)->map(function ($server) use ($user) {
            $serverPort = (string) $server->port;
            // 判断动态端口
            if (str_contains($serverPort, '-')) {
                $port = $serverPort;
                $server->port = (int) Helper::randomPort($port);
                $server->ports = $port;
            } else {
                $server->port = (int) $serverPort;
            }
            $server->password = $server->generateServerPassword($user);
            return $server;
        })->toArray();

        return $servers;
    }

    /**
     * Maintainable server_nodes mapped to legacy subscribe payload.
     */
    private static function getServerNodeAvailableServers(User $user): array
    {
        try {
            /** @var AccessControlService $accessControlService */
            $accessControlService = app(AccessControlService::class);
            $nodes = $accessControlService->getAccessibleNodesForUser($user);
        } catch (Throwable $e) {
            Log::warning('Failed to load accessible server nodes for subscribe', [
                'user_id' => $user->id,
                'message' => $e->getMessage(),
            ]);
            return [];
        }

        return $nodes
            ->filter(fn (ServerNode $node) => $node->isReportedOnline())
            ->map(fn(ServerNode $node) => self::mapServerNodeToLegacyServerPayload($node, $user))
            ->values()
            ->all();
    }

    private static function mapServerNodeToLegacyServerPayload(ServerNode $node, User $user): array
    {
        $normalizedType = Server::normalizeType((string) $node->protocol) ?? Server::TYPE_VMESS;
        $settings = is_array($node->settings) ? $node->settings : [];
        $protocolSettings = self::normalizeServerNodeProtocolSettings(
            $normalizedType,
            (string) $node->protocol,
            $settings
        );

        // Reuse legacy setting caster/password generator to keep protocol output compatible.
        $serverLike = new Server();
        $serverLike->type = $normalizedType;
        $serverLike->protocol_settings = $protocolSettings;
        $serverLike->created_at = self::resolveServerNodeCreatedAt($node);
        $password = $serverLike->generateServerPassword($user);

        $isOnline = $node->isReportedOnline() ? 1 : 0;
        $lastReportAt = $node->getLastReportAt();
        $lastUpdated = $lastReportAt
            ?? ($node->updated_at instanceof CarbonInterface ? $node->updated_at->timestamp : time());
        $subscribeNodeName = self::buildSubscribedServerNodeName($node);

        return [
            'id' => (int) $node->id,
            'type' => $normalizedType,
            'name' => $subscribeNodeName,
            'host' => (string) $node->host,
            // `port` is the user-facing subscribe port; `service_port` is node-side listen port.
            'port' => (int) $node->port,
            'service_port' => (int) $node->getEffectiveServicePort(),
            'rate' => 1,
            'tags' => ['server-node'],
            'protocol_settings' => $serverLike->protocol_settings,
            'password' => $password,
            'is_online' => $isOnline,
            'online' => 0,
            'available_status' => $isOnline ? Server::STATUS_ONLINE : Server::STATUS_OFFLINE,
            'cache_key' => sprintf('server_node:%d:%s', (int) $node->id, $normalizedType),
            'server_key' => null,
            'last_check_at' => $lastUpdated,
            'last_push_at' => $lastUpdated,
            'online_status' => $node->getOnlineStatus(),
        ];
    }

    private static function buildSubscribedServerNodeName(ServerNode $node): string
    {
        $ownerName = self::resolveServerNodeOwnerDisplayName($node);
        $nodeName = trim((string) $node->name);

        if ($ownerName === '') {
            return $nodeName !== '' ? $nodeName : '节点#' . (int) $node->id;
        }

        if ($nodeName === '') {
            return $ownerName;
        }

        return $ownerName . ' - ' . $nodeName;
    }

    private static function resolveServerNodeOwnerDisplayName(ServerNode $node): string
    {
        $owner = $node->owner;
        if (!$owner instanceof User) {
            return '';
        }

        $linuxDoUsername = trim((string) ($owner->linux_do_username ?? ''));
        if ($linuxDoUsername !== '') {
            return $linuxDoUsername;
        }

        $linuxDoName = trim((string) ($owner->linux_do_name ?? ''));
        if ($linuxDoName !== '') {
            return $linuxDoName;
        }

        $email = trim((string) ($owner->email ?? ''));
        if ($email !== '') {
            return $email;
        }

        return '用户#' . (int) $owner->id;
    }

    private static function resolveServerNodeCreatedAt(ServerNode $node): int
    {
        if ($node->created_at instanceof CarbonInterface) {
            return $node->created_at->timestamp;
        }

        $rawCreatedAt = $node->getRawOriginal('created_at');
        if (is_numeric($rawCreatedAt)) {
            return (int) $rawCreatedAt;
        }

        $parsed = strtotime((string) $rawCreatedAt);
        if ($parsed !== false) {
            return $parsed;
        }

        return time();
    }

    private static function normalizeServerNodeProtocolSettings(string $type, string $rawProtocol, array $settings): array
    {
        $defaults = match ($type) {
            Server::TYPE_VMESS => [
                'tls' => 0,
                'network' => 'tcp',
                'rules' => null,
                'network_settings' => [],
                'tls_settings' => [
                    'server_name' => null,
                    'allow_insecure' => false,
                ],
            ],
            Server::TYPE_VLESS => [
                'tls' => 0,
                'tls_settings' => [
                    'server_name' => null,
                    'allow_insecure' => false,
                ],
                'flow' => null,
                'network' => 'tcp',
                'network_settings' => [],
                'reality_settings' => [
                    'allow_insecure' => false,
                    'server_port' => null,
                    'server_name' => null,
                    'public_key' => null,
                    'private_key' => null,
                    'short_id' => null,
                ],
            ],
            Server::TYPE_TROJAN => [
                'allow_insecure' => false,
                'server_name' => null,
                'network' => 'tcp',
                'network_settings' => [],
            ],
            Server::TYPE_SHADOWSOCKS => [
                'cipher' => 'aes-128-gcm',
                'obfs' => null,
                'obfs_settings' => null,
                'plugin' => null,
                'plugin_opts' => null,
            ],
            Server::TYPE_HYSTERIA => [
                'version' => strtolower($rawProtocol) === ServerNode::PROTOCOL_HYSTERIA2
                    ? 2
                    : (int) ($settings['version'] ?? 2),
                'bandwidth' => [
                    'up' => null,
                    'down' => null,
                ],
                'obfs' => [
                    'open' => false,
                    'type' => 'salamander',
                    'password' => null,
                ],
                'tls' => [
                    'server_name' => null,
                    'allow_insecure' => false,
                ],
                'hop_interval' => null,
            ],
            Server::TYPE_TUIC => [
                'version' => 5,
                'congestion_control' => 'cubic',
                'alpn' => ['h3'],
                'udp_relay_mode' => 'native',
                'tls' => [
                    'server_name' => null,
                    'allow_insecure' => false,
                ],
            ],
            Server::TYPE_ANYTLS => [
                'padding_scheme' => [
                    'stop=8',
                    '0=30-30',
                    '1=100-400',
                    '2=400-500,c,500-1000,c,500-1000,c,500-1000,c,500-1000',
                    '3=9-9,500-1000',
                    '4=500-1000',
                    '5=500-1000',
                    '6=500-1000',
                    '7=500-1000',
                ],
                'tls' => [
                    'server_name' => null,
                    'allow_insecure' => false,
                ],
            ],
            Server::TYPE_SOCKS => [
                'tls' => 0,
                'tls_settings' => [
                    'allow_insecure' => false,
                    'server_name' => null,
                ],
                'udp_over_tcp' => false,
            ],
            Server::TYPE_HTTP => [
                'tls' => 0,
                'tls_settings' => [
                    'allow_insecure' => false,
                    'server_name' => null,
                ],
                'path' => null,
                'headers' => null,
            ],
            Server::TYPE_NAIVE => [
                'tls' => 0,
                'tls_settings' => [
                    'allow_insecure' => false,
                    'server_name' => null,
                ],
            ],
            Server::TYPE_MIERU => [
                'protocol' => 0,
                'transport' => 'tcp',
                'multiplexing' => 'MULTIPLEXING_LOW',
            ],
            default => [],
        };

        $merged = array_replace_recursive($defaults, $settings);

        if ($type === Server::TYPE_HYSTERIA && strtolower($rawProtocol) === ServerNode::PROTOCOL_HYSTERIA2) {
            $merged['version'] = 2;
        }

        return $merged;
    }

    /**
     * 根据权限组获取可用的用户列表
     * @param array $groupIds
     * @return Collection
     */
    public static function getAvailableUsers(Server $node)
    {
        $credentialService = app(SubscriptionCredentialService::class);

        if (!$node->isActive() || $node->isTrafficExceeded()) {
            return collect();
        }

        $users = User::query()
            ->whereIn('group_id', $node->group_ids)
            ->whereRaw('u + d < transfer_enable')
            ->where(function ($query) {
                $query->where('expired_at', '>=', time())
                    ->orWhere('expired_at', NULL);
            })
            ->where('banned', 0)
            ->select([
                'id',
                'uuid',
                'subscription_credential_version',
                'speed_limit',
                'device_limit'
            ])
            ->get();

        $users->each(function (User $user) use ($credentialService) {
            $user->uuid = $credentialService->getEffectiveUuid($user);
        });

        return HookManager::filter('server.users.get', $users, $node);
    }

    // 获取路由规则
    public static function getRoutes(array $routeIds)
    {
        $routes = ServerRoute::select(['id', 'match', 'action', 'action_value'])->whereIn('id', $routeIds)->get();
        return $routes;
    }

    /**
     * 根据协议类型和标识获取服务器
     * @param int $serverId
     * @param string $serverType
     * @return Server|null
     */
    public static function getServer($serverId, ?string $serverType)
    {
        return Server::query()
            ->when($serverType, function ($query) use ($serverType) {
                $query->where('type', Server::normalizeType($serverType));
            })
            ->where(function ($query) use ($serverId) {
                $query->where('code', $serverId)
                    ->orWhere('id', $serverId);
            })
            ->orderByRaw('CASE WHEN code = ? THEN 0 ELSE 1 END', [$serverId])
            ->first();
    }
}
