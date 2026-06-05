<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\Server;
use App\Models\ServerNode;
use App\Models\User;
use App\Services\AccessControlService;
use App\Services\LimitControlService;
use App\Support\PanelUrlResolver;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Validator;
use Illuminate\Validation\Rule;

class ServerNodeController extends Controller
{
    protected AccessControlService $accessControlService;
    protected LimitControlService $limitControlService;

    public function __construct(
        AccessControlService $accessControlService,
        LimitControlService $limitControlService
    ) {
        $this->accessControlService = $accessControlService;
        $this->limitControlService = $limitControlService;
    }

    /**
     * 获取后端支持的节点协议列表（供前端动态渲染）
     */
    public function protocols(Request $request): JsonResponse
    {
        $labels = [
            ServerNode::PROTOCOL_VMESS => 'VMess',
            ServerNode::PROTOCOL_VLESS => 'VLESS',
            ServerNode::PROTOCOL_TROJAN => 'Trojan',
            ServerNode::PROTOCOL_SHADOWSOCKS => 'Shadowsocks',
            ServerNode::PROTOCOL_HYSTERIA => 'Hysteria',
            ServerNode::PROTOCOL_HYSTERIA2 => 'Hysteria2',
            ServerNode::PROTOCOL_TUIC => 'TUIC',
            ServerNode::PROTOCOL_ANYTLS => 'AnyTLS',
            ServerNode::PROTOCOL_SOCKS => 'SOCKS',
            ServerNode::PROTOCOL_HTTP => 'HTTP',
            ServerNode::PROTOCOL_NAIVE => 'Naive',
            ServerNode::PROTOCOL_MIERU => 'Mieru',
        ];

        $items = collect(ServerNode::SUPPORTED_PROTOCOLS)
            ->map(fn (string $protocol) => [
                'value' => $protocol,
                'label' => $labels[$protocol] ?? strtoupper($protocol),
                'template' => $this->buildProtocolTemplate($protocol),
            ])
            ->values()
            ->all();

        return response()->json([
            'data' => $items,
        ]);
    }

    private function buildProtocolTemplate(string $protocol): array
    {
        $normalizedType = Server::normalizeType($protocol) ?? Server::TYPE_VMESS;
        $template = Server::getProtocolSettingTemplateForType($normalizedType);

        // Hysteria2 shares type with hysteria, but default should be version=2.
        if (strtolower($protocol) === ServerNode::PROTOCOL_HYSTERIA2) {
            $template['version'] = 2;
        }

        return is_array($template) ? $template : [];
    }

    /**
     * 获取用户的服务器节点列表
     */
    public function index(Request $request): JsonResponse
    {
        $user = Auth::user();
        
        $nodes = ServerNode::where('user_id', $user->id)
            ->with(['auditRules', 'onlineSessions', 'trafficRecords'])
            ->orderBy('created_at', 'desc')
            ->get()
            ->map(function ($node) {
                $tcpingMonitorable = $node->isTcpingMonitorable();
                return [
                    'id' => $node->id,
                    'name' => $node->name,
                    'host' => $node->host,
                    'port' => $node->port,
                    'service_port' => $node->service_port,
                    'effective_service_port' => $node->getEffectiveServicePort(),
                    'protocol' => $node->protocol,
                    'location_code' => $node->location_code,
                    'location_name' => $node->location_name,
                    'status' => $node->status,
                    'online_status' => $node->getOnlineStatus(),
                    'is_online' => $node->isReportedOnline(),
                    'last_report_at' => $node->getLastReportAt(),
                    'traffic_limit' => $node->traffic_limit,
                    'traffic_used' => $node->traffic_used,
                    'traffic_usage_percentage' => $node->getTrafficUsagePercentage(),
                    'remaining_traffic' => $node->getRemainingTraffic(),
                    'traffic_multiplier' => $node->getEffectiveTrafficMultiplier(),
                    'online_users' => $node->getOnlineUserCount(),
                    'device_limit' => $node->device_limit,
                    'connection_limit' => $node->connection_limit,
                    'speed_limit_up' => $node->speed_limit_up,
                    'speed_limit_down' => $node->speed_limit_down,
                    'cross_node_ip_limit' => $node->cross_node_ip_limit,
                    'concurrent_ip_limit' => $node->concurrent_ip_limit,
                    'tcping_enabled' => $tcpingMonitorable,
                    'tcping_host' => $node->tcping_host,
                    'tcping_port' => $node->tcping_port,
                    'tcping_interval_seconds' => $node->tcping_interval_seconds,
                    'tcping_timeout_ms' => $node->tcping_timeout_ms,
                    'tcping_alert_after_seconds' => $node->tcping_alert_after_seconds,
                    'tcping_recover_after_seconds' => $node->tcping_recover_after_seconds,
                    'tcping_status' => $tcpingMonitorable ? ($node->tcping_last_status ?: 'unknown') : 'unsupported',
                    'tcping_last_latency_ms' => $tcpingMonitorable ? $node->tcping_last_latency_ms : null,
                    'tcping_last_sampled_at' => $tcpingMonitorable ? $node->tcping_last_sampled_at : null,
                    'is_active' => $node->isActive(),
                    'is_traffic_exceeded' => $node->isTrafficExceeded(),
                    'created_at' => $node->created_at,
                    'updated_at' => $node->updated_at,
                ];
            });

        return response()->json([
            'data' => $nodes
        ]);
    }

    /**
     * 创建新的服务器节点
     */
    public function store(Request $request): JsonResponse
    {
        $user = Auth::user();

        $validator = Validator::make($request->all(), [
            'name' => 'required|string|max:255',
            'host' => 'required|string|max:255',
            'port' => 'required|integer|min:1|max:65535',
            'service_port' => 'nullable|integer|min:1|max:65535',
            'status' => 'prohibited',
            'protocol' => ['required', Rule::in(ServerNode::SUPPORTED_PROTOCOLS)],
            'location_code' => 'required|string|max:16',
            'location_name' => 'required|string|max:128',
            'settings' => 'nullable|array',
            'traffic_limit' => 'nullable|integer|min:0',
            'traffic_multiplier' => 'nullable|numeric|min:0.1|max:100',
            'device_limit' => 'nullable|integer|min:0',
            'connection_limit' => 'nullable|integer|min:0',
            'speed_limit_up' => 'nullable|integer|min:0',
            'speed_limit_down' => 'nullable|integer|min:0',
            'cross_node_ip_limit' => 'nullable|integer|min:0',
            'concurrent_ip_limit' => 'nullable|integer|min:0',
            'tcping_enabled' => 'nullable|boolean',
            'tcping_host' => 'nullable|string|max:255',
            'tcping_port' => 'nullable|integer|min:1|max:65535',
            'tcping_interval_seconds' => 'nullable|integer|min:15|max:3600',
            'tcping_timeout_ms' => 'nullable|integer|min:500|max:60000',
            'tcping_alert_after_seconds' => 'nullable|integer|min:60|max:86400',
            'tcping_recover_after_seconds' => 'nullable|integer|min:30|max:86400',
            'access_control' => 'nullable|array',
            'access_control.min_trust_level' => 'nullable|integer|min:0|max:4',
            'access_control.authorized_users' => 'nullable|array',
            'access_control.authorized_users.*' => 'integer|exists:v2_user,id',
            'access_control.free_quota_gb_by_trust_level' => 'nullable|array',
            'access_control.free_quota_gb_by_trust_level.*' => 'numeric|min:0',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'message' => 'Validation failed',
                'errors' => $validator->errors()
            ], 422);
        }

        $nodeData = $validator->validated();
        if (array_key_exists('service_port', $nodeData)) {
            $nodeData['service_port'] = $nodeData['service_port'] ?: null;
        }
        $nodeData['user_id'] = $user->id;
        $nodeData['status'] = ServerNode::STATUS_INACTIVE;
        $nodeData['traffic_used'] = 0;
        $nodeData['traffic_multiplier'] = max(0.1, (float) ($nodeData['traffic_multiplier'] ?? 1));
        // TCPing 监控由系统统一开启，节点创建/编辑不再允许关闭。
        $nodeData['tcping_enabled'] = true;
        $nodeData['tcping_host'] = trim((string) ($nodeData['tcping_host'] ?? '')) ?: $nodeData['host'];
        $protocol = strtolower((string) ($nodeData['protocol'] ?? ''));
        $isUdpProtocol = in_array($protocol, [
            ServerNode::PROTOCOL_HYSTERIA,
            ServerNode::PROTOCOL_HYSTERIA2,
            ServerNode::PROTOCOL_TUIC,
        ], true);
        if ($isUdpProtocol) {
            // UDP 节点：未配置 TCPing 探测端口则不下发探测目标。
            $nodeData['tcping_port'] = array_key_exists('tcping_port', $nodeData) && $nodeData['tcping_port'] !== null
                ? (int) $nodeData['tcping_port']
                : null;
        } else {
            // TCP 协议节点：留空默认跟随访问端口，打开即用。
            $nodeData['tcping_port'] = array_key_exists('tcping_port', $nodeData) && $nodeData['tcping_port'] !== null
                ? (int) $nodeData['tcping_port']
                : (int) $nodeData['port'];
        }
        $nodeData['tcping_interval_seconds'] = max(15, (int) ($nodeData['tcping_interval_seconds'] ?? 60));
        $nodeData['tcping_timeout_ms'] = max(500, (int) ($nodeData['tcping_timeout_ms'] ?? 3000));
        $nodeData['tcping_alert_after_seconds'] = max(60, (int) ($nodeData['tcping_alert_after_seconds'] ?? 300));
        $nodeData['tcping_recover_after_seconds'] = max(30, (int) ($nodeData['tcping_recover_after_seconds'] ?? 120));

        // 只有超级管理员可以设置跨节点并发IP限制
        if (isset($nodeData['concurrent_ip_limit']) && !$user->is_super_admin) {
            unset($nodeData['concurrent_ip_limit']);
        }

        $node = ServerNode::create($nodeData);

        // 设置访问控制
        if (isset($nodeData['access_control'])) {
            $this->accessControlService->setAccessControl($node, $nodeData['access_control']);
        }

        return response()->json([
            'message' => 'Server node created successfully',
            'data' => $node->load(['auditRules', 'onlineSessions'])
        ], 201);
    }

    /**
     * 获取指定服务器节点详情
     */
    public function show(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        
        $node = ServerNode::where('id', $id)
            ->where('user_id', $user->id)
            ->with(['auditRules', 'onlineSessions', 'trafficRecords', 'authorizedUsers'])
            ->first();

        if (!$node) {
            return response()->json([
                'message' => 'Server node not found'
            ], 404);
        }

        return response()->json([
            'data' => [
                'id' => $node->id,
                'name' => $node->name,
                'host' => $node->host,
                'port' => $node->port,
                'service_port' => $node->service_port,
                'effective_service_port' => $node->getEffectiveServicePort(),
                'protocol' => $node->protocol,
                'location_code' => $node->location_code,
                'location_name' => $node->location_name,
                'settings' => $node->settings,
                'status' => $node->status,
                'online_status' => $node->getOnlineStatus(),
                'is_online' => $node->isReportedOnline(),
                'last_report_at' => $node->getLastReportAt(),
                'traffic_limit' => $node->traffic_limit,
                'traffic_used' => $node->traffic_used,
                'traffic_usage_percentage' => $node->getTrafficUsagePercentage(),
                'remaining_traffic' => $node->getRemainingTraffic(),
                'traffic_multiplier' => $node->getEffectiveTrafficMultiplier(),
                'access_control' => $node->access_control,
                'device_limit' => $node->device_limit,
                'connection_limit' => $node->connection_limit,
                'speed_limit_up' => $node->speed_limit_up,
                'speed_limit_down' => $node->speed_limit_down,
                'cross_node_ip_limit' => $node->cross_node_ip_limit,
                'concurrent_ip_limit' => $node->concurrent_ip_limit,
                'tcping_enabled' => $node->isTcpingMonitorable(),
                'tcping_host' => $node->tcping_host,
                'tcping_port' => $node->tcping_port,
                'tcping_interval_seconds' => $node->tcping_interval_seconds,
                'tcping_timeout_ms' => $node->tcping_timeout_ms,
                'tcping_alert_after_seconds' => $node->tcping_alert_after_seconds,
                'tcping_recover_after_seconds' => $node->tcping_recover_after_seconds,
                'tcping_status' => $node->isTcpingMonitorable() ? ($node->tcping_last_status ?: 'unknown') : 'unsupported',
                'tcping_last_latency_ms' => $node->isTcpingMonitorable() ? $node->tcping_last_latency_ms : null,
                'tcping_last_error' => $node->isTcpingMonitorable() ? $node->tcping_last_error : null,
                'tcping_last_sampled_at' => $node->isTcpingMonitorable() ? $node->tcping_last_sampled_at : null,
                'v2bx_node_id' => $node->v2bx_node_id,
                'v2bx_config' => $node->v2bx_config,
                'online_users' => $node->getOnlineUserCount(),
                'audit_rules' => $node->auditRules,
                'authorized_users' => $node->authorizedUsers,
                'is_active' => $node->isActive(),
                'is_traffic_exceeded' => $node->isTrafficExceeded(),
                'created_at' => $node->created_at,
                'updated_at' => $node->updated_at,
            ]
        ]);
    }

    /**
     * 更新服务器节点
     */
    public function update(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        
        $node = ServerNode::where('id', $id)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json([
                'message' => 'Server node not found'
            ], 404);
        }

        $validator = Validator::make($request->all(), [
            'name' => 'sometimes|string|max:255',
            'host' => 'sometimes|string|max:255',
            'port' => 'sometimes|integer|min:1|max:65535',
            'service_port' => 'nullable|integer|min:1|max:65535',
            'status' => 'prohibited',
            'protocol' => ['sometimes', Rule::in(ServerNode::SUPPORTED_PROTOCOLS)],
            'location_code' => 'sometimes|required_with:location_name|string|max:16',
            'location_name' => 'sometimes|required_with:location_code|string|max:128',
            'settings' => 'nullable|array',
            'traffic_limit' => 'nullable|integer|min:0',
            'traffic_multiplier' => 'nullable|numeric|min:0.1|max:100',
            'device_limit' => 'nullable|integer|min:0',
            'connection_limit' => 'nullable|integer|min:0',
            'speed_limit_up' => 'nullable|integer|min:0',
            'speed_limit_down' => 'nullable|integer|min:0',
            'cross_node_ip_limit' => 'nullable|integer|min:0',
            'concurrent_ip_limit' => 'nullable|integer|min:0',
            'tcping_enabled' => 'nullable|boolean',
            'tcping_host' => 'nullable|string|max:255',
            'tcping_port' => 'nullable|integer|min:1|max:65535',
            'tcping_interval_seconds' => 'nullable|integer|min:15|max:3600',
            'tcping_timeout_ms' => 'nullable|integer|min:500|max:60000',
            'tcping_alert_after_seconds' => 'nullable|integer|min:60|max:86400',
            'tcping_recover_after_seconds' => 'nullable|integer|min:30|max:86400',
            'access_control' => 'nullable|array',
            'access_control.min_trust_level' => 'nullable|integer|min:0|max:4',
            'access_control.authorized_users' => 'nullable|array',
            'access_control.authorized_users.*' => 'integer|exists:v2_user,id',
            'access_control.free_quota_gb_by_trust_level' => 'nullable|array',
            'access_control.free_quota_gb_by_trust_level.*' => 'numeric|min:0',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'message' => 'Validation failed',
                'errors' => $validator->errors()
            ], 422);
        }

        $updateData = $validator->validated();
        if (array_key_exists('service_port', $updateData)) {
            $updateData['service_port'] = $updateData['service_port'] ?: null;
        }
        if (array_key_exists('traffic_multiplier', $updateData)) {
            $updateData['traffic_multiplier'] = max(0.1, (float) $updateData['traffic_multiplier']);
        }
        // TCPing 监控由系统统一开启，节点编辑不再允许关闭。
        $updateData['tcping_enabled'] = true;
        if (array_key_exists('tcping_interval_seconds', $updateData)) {
            $updateData['tcping_interval_seconds'] = max(15, (int) $updateData['tcping_interval_seconds']);
        }
        if (array_key_exists('tcping_timeout_ms', $updateData)) {
            $updateData['tcping_timeout_ms'] = max(500, (int) $updateData['tcping_timeout_ms']);
        }
        if (array_key_exists('tcping_alert_after_seconds', $updateData)) {
            $updateData['tcping_alert_after_seconds'] = max(60, (int) $updateData['tcping_alert_after_seconds']);
        }
        if (array_key_exists('tcping_recover_after_seconds', $updateData)) {
            $updateData['tcping_recover_after_seconds'] = max(30, (int) $updateData['tcping_recover_after_seconds']);
        }
        if (array_key_exists('tcping_host', $updateData)) {
            $updateData['tcping_host'] = trim((string) ($updateData['tcping_host'] ?? '')) ?: ($updateData['host'] ?? $node->host);
        } elseif (array_key_exists('host', $updateData) && (($node->tcping_host ?? '') === '' || (string) $node->tcping_host === (string) $node->host)) {
            $updateData['tcping_host'] = (string) $updateData['host'];
        }
        $effectiveProtocol = strtolower((string) ($updateData['protocol'] ?? $node->protocol));
        $isUdpProtocol = in_array($effectiveProtocol, [
            ServerNode::PROTOCOL_HYSTERIA,
            ServerNode::PROTOCOL_HYSTERIA2,
            ServerNode::PROTOCOL_TUIC,
        ], true);
        if (array_key_exists('tcping_port', $updateData)) {
            if ($updateData['tcping_port'] === null) {
                $updateData['tcping_port'] = $isUdpProtocol ? null : (int) ($updateData['port'] ?? $node->port);
            } else {
                $updateData['tcping_port'] = (int) $updateData['tcping_port'];
            }
        } elseif (!$isUdpProtocol && array_key_exists('port', $updateData)) {
            // 非 UDP 节点：若探测端口此前跟随访问端口，则在修改访问端口时保持同步。
            if ($node->tcping_port === null || (int) $node->tcping_port === (int) $node->port) {
                $updateData['tcping_port'] = (int) $updateData['port'];
            }
        }

        // 只有超级管理员可以设置跨节点并发IP限制
        if (isset($updateData['concurrent_ip_limit']) && !$user->is_super_admin) {
            unset($updateData['concurrent_ip_limit']);
        }

        $node->update($updateData);

        // 更新访问控制
        if (isset($updateData['access_control'])) {
            $this->accessControlService->setAccessControl($node, $updateData['access_control']);
        }

        return response()->json([
            'message' => 'Server node updated successfully',
            'data' => $node->fresh()->load(['auditRules', 'onlineSessions'])
        ]);
    }

    /**
     * 删除服务器节点
     */
    public function destroy(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        
        $node = ServerNode::where('id', $id)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json([
                'message' => 'Server node not found'
            ], 404);
        }

        // 检查是否有在线用户
        $onlineUsers = $node->getOnlineUserCount();
        if ($onlineUsers > 0) {
            return response()->json([
                'message' => 'Cannot delete node with active users',
                'online_users' => $onlineUsers
            ], 409);
        }

        $node->delete();

        return response()->json([
            'message' => 'Server node deleted successfully'
        ]);
    }

    /**
     * 部署服务器节点
     */
    public function deploy(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        
        $node = ServerNode::where('id', $id)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json([
                'message' => 'Server node not found'
            ], 404);
        }

        if (!ServerNode::supportsV2bxDeploy($node->protocol)) {
            return response()->json([
                'message' => '当前协议暂不支持 V2bX 一键部署',
                'data' => [
                    'protocol' => strtolower((string) $node->protocol),
                    'supported_protocols' => ServerNode::V2BX_SUPPORTED_PROTOCOLS,
                ],
            ], 422);
        }

        // 更新节点状态为部署中
        $node->update(['status' => ServerNode::STATUS_DEPLOYING]);

        // TODO: 实际的部署逻辑，这里可以集成 V2bX 或其他代理服务器部署
        // 暂时模拟部署成功
        $node->update([
            'status' => ServerNode::STATUS_ACTIVE,
            'v2bx_node_id' => $node->id, // 临时使用节点ID作为V2bX节点ID
        ]);

        return response()->json([
            'message' => 'Server node deployed successfully',
            'data' => $node->fresh()
        ]);
    }

    /**
     * 获取 V2bX 一键部署命令（前端可直接复制）
     */
    public function deployCommand(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();

        $node = ServerNode::where('id', $id)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json([
                'message' => 'Server node not found'
            ], 404);
        }

        if (!ServerNode::supportsV2bxDeploy($node->protocol)) {
            return response()->json([
                'message' => '当前协议暂不支持 V2bX 一键部署命令',
                'data' => [
                    'protocol' => strtolower((string) $node->protocol),
                    'supported_protocols' => ServerNode::V2BX_SUPPORTED_PROTOCOLS,
                ],
            ], 422);
        }

        // 统一节点对接 token 与用户 API Key，避免面板显示与节点配置不一致
        if (empty($user->api_key)) {
            $user->generateApiKey();
            $user->refresh();
        }

        $token = (string) $user->api_key;
        if ($token === '') {
            return response()->json([
                'message' => 'API Key unavailable'
            ], 422);
        }

        if ($node->v2bx_token !== $token) {
            $node->v2bx_token = $token;
            $node->save();
        }

        $panelUrl = $this->resolvePanelUrl($request);
        $scriptVersion = (string) (@filemtime(public_path('v2bx-install.sh')) ?: time());
        $scriptUrl = $panelUrl . '/v2bx-install.sh?v=' . rawurlencode($scriptVersion);
        $nodeId = $node->v2bx_node_id ?? $node->id;
        $nodeType = strtolower($node->protocol);
        $certDomain = trim((string) ($node->host ?? ''));
        $autoCertMode = ServerNode::prefersAutoCertForV2bx($nodeType);

        $core = null;
        if (is_array($node->v2bx_config)) {
            $core = $node->v2bx_config['core'] ?? ($node->v2bx_config['Core'] ?? null);
        }

        $command = sprintf(
            "curl -fsSL %s | bash -s -- --panel %s --node-id %s --node-type %s --token %s%s",
            escapeshellarg($scriptUrl),
            escapeshellarg($panelUrl),
            escapeshellarg((string) $nodeId),
            escapeshellarg($nodeType),
            escapeshellarg($token),
            $core ? ' --core ' . escapeshellarg((string) $core) : ''
        );
        if ($autoCertMode) {
            $command .= ' --cert-mode self';
            if ($certDomain !== '') {
                $command .= ' --cert-domain ' . escapeshellarg($certDomain);
            }
        }

        return response()->json([
            'data' => [
                'script_url' => $scriptUrl,
                'command' => $command,
                'panel_url' => $panelUrl,
                'node_id' => $nodeId,
                'node_type' => $nodeType,
                'token' => $token,
                'core' => $core,
                'cert_mode' => $autoCertMode ? 'self' : null,
                'cert_domain' => $autoCertMode && $certDomain !== '' ? $certDomain : null,
            ]
        ]);
    }

    private function resolvePanelUrl(Request $request): string
    {
        return PanelUrlResolver::fromRequest($request);
    }

    /**
     * 获取服务器节点状态
     */
    public function status(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        
        $node = ServerNode::where('id', $id)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json([
                'message' => 'Server node not found'
            ], 404);
        }

        return response()->json([
            'data' => [
                'id' => $node->id,
                'name' => $node->name,
                'status' => $node->status,
                'online_status' => $node->getOnlineStatus(),
                'is_online' => $node->isReportedOnline(),
                'last_report_at' => $node->getLastReportAt(),
                'is_active' => $node->isActive(),
                'online_users' => $node->getOnlineUserCount(),
                'traffic_usage_percentage' => $node->getTrafficUsagePercentage(),
                'is_traffic_exceeded' => $node->isTrafficExceeded(),
                'tcping_status' => $node->tcping_last_status ?: 'unknown',
                'tcping_last_latency_ms' => $node->tcping_last_latency_ms,
                'tcping_last_error' => $node->tcping_last_error,
                'tcping_last_sampled_at' => $node->tcping_last_sampled_at,
                'last_check_at' => $node->updated_at,
            ]
        ]);
    }

    /**
     * 配置访问控制
     */
    public function configureAccess(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        
        $node = ServerNode::where('id', $id)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json([
                'message' => 'Server node not found'
            ], 404);
        }

        $validator = Validator::make($request->all(), [
            'min_trust_level' => 'nullable|integer|min:0|max:4',
            'authorized_users' => 'nullable|array',
            'authorized_users.*' => 'integer|exists:v2_user,id',
            'free_quota_gb_by_trust_level' => 'nullable|array',
            'free_quota_gb_by_trust_level.*' => 'numeric|min:0',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'message' => 'Validation failed',
                'errors' => $validator->errors()
            ], 422);
        }

        $accessControl = $validator->validated();
        
        $this->accessControlService->setAccessControl($node, $accessControl);

        return response()->json([
            'message' => 'Access control configured successfully',
            'data' => $node->fresh(['authorizedUsers'])
        ]);
    }
}
