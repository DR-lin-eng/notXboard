<?php


namespace App\Http\Middleware;

use App\Exceptions\ApiException;
use App\Models\Server as LegacyServer;
use App\Models\ServerNode;
use App\Services\ServerService;
use Closure;
use Illuminate\Http\Request;

class Server
{
    public function handle(Request $request, Closure $next, ?string $nodeType = null)
    {
        $this->validateRequest($request);
        $token = (string) $request->input('token');
        $nodeId = $request->input('node_id');

        $serverNode = $this->findServerNodeByDirectToken($nodeId, $token)
            ?? $this->findLegacyServerByToken($nodeId, $token, $nodeType);

        if (!$serverNode) {
            throw new ApiException('Invalid server token');
        }

        $request->attributes->set('node_info', $serverNode);
        $request->attributes->set(
            'node_kind',
            $serverNode instanceof ServerNode ? 'server_node' : 'legacy_server'
        );
        return $next($request);
    }

    private function findServerNodeByDirectToken(mixed $nodeId, string $token): ?ServerNode
    {
        return ServerNode::query()
            ->where(function ($query) use ($nodeId) {
                $query->where('v2bx_node_id', $nodeId)
                    ->orWhere('id', $nodeId);
            })
            ->where('v2bx_token', $token)
            ->first();
    }

    private function findLegacyServerByToken(mixed $nodeId, string $token, ?string $nodeType): ?LegacyServer
    {
        $legacyToken = trim((string) admin_setting('server_token', ''));
        if ($legacyToken === '' || !hash_equals($legacyToken, $token)) {
            return null;
        }

        return ServerService::getServer($nodeId, $nodeType);
    }

    private function validateRequest(Request $request): void
    {
        $request->validate([
            'token' => 'required|string',
            'node_id' => 'required',
            'node_type' => 'nullable'
        ]);
    }
}
