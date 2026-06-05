<?php

namespace App\Http\Controllers\V1\Tcping;

use App\Http\Controllers\Controller;
use App\Models\TcpingAgent;
use App\Services\TcpingService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class AgentController extends Controller
{
    public function __construct(
        private readonly TcpingService $tcpingService,
    ) {
    }

    public function config(Request $request): JsonResponse
    {
        $agent = $this->resolveAgent($request);
        if (!$agent) {
            return response()->json(['success' => false, 'error' => 'Invalid TCPing agent token'], 401);
        }

        return response()->json([
            'success' => true,
            'data' => $this->tcpingService->buildAgentConfig($agent),
        ]);
    }

    public function heartbeat(Request $request): JsonResponse
    {
        $agent = $this->resolveAgent($request);
        if (!$agent) {
            return response()->json(['success' => false, 'error' => 'Invalid TCPing agent token'], 401);
        }

        $this->tcpingService->touchHeartbeat($agent);

        return response()->json([
            'success' => true,
            'data' => true,
        ]);
    }

    public function samples(Request $request): JsonResponse
    {
        $agent = $this->resolveAgent($request);
        if (!$agent) {
            return response()->json(['success' => false, 'error' => 'Invalid TCPing agent token'], 401);
        }

        $payload = $request->json()->all();
        $samples = $payload['samples'] ?? $payload;
        if (!is_array($samples)) {
            return response()->json(['success' => false, 'error' => 'Invalid samples payload'], 422);
        }

        $result = $this->tcpingService->ingestSamples($agent, $samples);

        return response()->json([
            'success' => true,
            'data' => $result,
        ]);
    }

    private function resolveAgent(Request $request): ?TcpingAgent
    {
        $token = trim((string) (
            $request->bearerToken()
            ?: $request->query('token')
            ?: $request->input('token')
        ));

        if ($token === '') {
            return null;
        }

        return $this->tcpingService->findAgentByToken($token);
    }
}
