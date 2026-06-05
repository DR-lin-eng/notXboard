<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\ServerNode;
use App\Models\TcpingAgent;
use App\Models\User;
use App\Services\AccessControlService;
use App\Services\TcpingService;
use App\Support\PanelUrlResolver;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class TcpingController extends Controller
{
    public function __construct(
        private readonly TcpingService $tcpingService,
        private readonly AccessControlService $accessControlService,
    ) {
    }

    public function agents(Request $request): JsonResponse
    {
        /** @var User $user */
        $user = $request->user();

        $items = $this->tcpingService->getAgentsForUser($user)
            ->map(fn (TcpingAgent $agent) => $this->tcpingService->serializeAgent($agent))
            ->values()
            ->all();

        return response()->json([
            'success' => true,
            'data' => $items,
        ]);
    }

    public function createAgent(Request $request): JsonResponse
    {
        /** @var User $user */
        $user = $request->user();

        $data = $request->validate([
            'name' => 'required|string|max:128',
            'location_code' => 'required|string|max:16',
            'location_name' => 'required|string|max:128',
            'location_province' => 'nullable|string|max:64',
        ]);

        $locationCode = strtoupper(trim((string) $data['location_code']));
        $locationName = trim((string) $data['location_name']);
        $province = trim((string) ($data['location_province'] ?? ''));

        if ($locationCode === 'CN' && $province === '') {
            return response()->json([
                'success' => false,
                'error' => '中国大陆地区的探针必须填写省份归属',
            ], 422);
        }

        $agent = $this->tcpingService->createAgentForUser(
            $user,
            $data['name'],
            $locationCode,
            $locationName,
            $province !== '' ? $province : null
        );

        return response()->json([
            'success' => true,
            'data' => $this->tcpingService->serializeAgent($agent),
        ]);
    }

    public function rotateToken(Request $request, int $id): JsonResponse
    {
        /** @var User $user */
        $user = $request->user();

        $agent = TcpingAgent::query()
            ->whereKey($id)
            ->where('user_id', $user->id)
            ->first();

        if (!$agent) {
            return response()->json(['success' => false, 'error' => 'TCPing agent not found'], 404);
        }

        $agent->rotateToken();
        $agent->save();

        return response()->json([
            'success' => true,
            'data' => $this->tcpingService->serializeAgent($agent),
        ]);
    }

    public function toggle(Request $request, int $id): JsonResponse
    {
        /** @var User $user */
        $user = $request->user();

        $agent = TcpingAgent::query()
            ->whereKey($id)
            ->where('user_id', $user->id)
            ->first();

        if (!$agent) {
            return response()->json(['success' => false, 'error' => 'TCPing agent not found'], 404);
        }

        return response()->json([
            'success' => false,
            'error' => 'TCPing Agent 监控由系统统一调度，当前不允许手动停用/启用',
        ], 403);
    }

    public function installCommand(Request $request, int $id): JsonResponse
    {
        /** @var User $user */
        $user = $request->user();

        $agent = TcpingAgent::query()
            ->whereKey($id)
            ->where('user_id', $user->id)
            ->first();

        if (!$agent) {
            return response()->json(['success' => false, 'error' => 'TCPing agent not found'], 404);
        }

        $panelUrl = $this->resolvePanelUrl($request);
        $scriptVersion = (string) (@filemtime(public_path('tcping-agent-install.sh')) ?: time());
        $scriptUrl = $panelUrl . '/tcping-agent-install.sh?v=' . rawurlencode($scriptVersion);

        return response()->json([
            'success' => true,
            'data' => [
                'panel_url' => $panelUrl,
                'script_url' => $scriptUrl,
                'token' => (string) $agent->token,
                'command' => sprintf(
                    'curl -fsSL %s | bash -s -- --panel %s --token %s',
                    escapeshellarg($scriptUrl),
                    escapeshellarg($panelUrl),
                    escapeshellarg((string) $agent->token)
                ),
            ],
        ]);
    }

    public function nodeOverview(Request $request, int $id): JsonResponse
    {
        /** @var User $user */
        $user = $request->user();

        $node = ServerNode::query()->whereKey($id)->first();
        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        if (!$this->canUserViewNode($user, $node)) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $hours = max(1, min(24 * 30, (int) $request->query('hours', 24)));
        $payload = $this->tcpingService->getNodeOverviewForUser($user, $node, $hours);
        $payload['can_manage'] = (bool) ($user->is_super_admin || (int) $node->user_id === (int) $user->id);

        return response()->json([
            'success' => true,
            'data' => $payload,
        ]);
    }

    private function canUserViewNode(User $user, ServerNode $node): bool
    {
        if ((bool) $user->is_super_admin) {
            return true;
        }

        if ((int) $node->user_id === (int) $user->id) {
            return true;
        }

        return $this->accessControlService->canUserAccessNode($user, $node);
    }

    private function resolvePanelUrl(Request $request): string
    {
        return PanelUrlResolver::fromRequest($request);
    }
}
