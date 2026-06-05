<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\AuditRule;
use App\Models\ServerNode;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Validation\Rule;

class AuditRuleController extends Controller
{
    public function index(Request $request, int $nodeId): JsonResponse
    {
        $user = Auth::user();

        $node = ServerNode::query()
            ->whereKey($nodeId)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $rules = AuditRule::query()
            ->where('node_id', $node->id)
            ->orderByDesc('id')
            ->get();

        return response()->json(['success' => true, 'data' => $rules]);
    }

    public function store(Request $request, int $nodeId): JsonResponse
    {
        $user = Auth::user();

        $node = ServerNode::query()
            ->whereKey($nodeId)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $data = $request->validate([
            'rule_type' => ['required', Rule::in([AuditRule::TYPE_DOMAIN, AuditRule::TYPE_PROTOCOL, AuditRule::TYPE_IP])],
            'rule_pattern' => 'required|string|max:500',
            'action' => ['required', Rule::in([AuditRule::ACTION_BLOCK, AuditRule::ACTION_ALLOW, AuditRule::ACTION_LOG])],
            'is_active' => 'sometimes|boolean',
        ]);

        $rule = AuditRule::create([
            ...$data,
            'node_id' => $node->id,
            'is_active' => $data['is_active'] ?? true,
        ]);

        return response()->json(['success' => true, 'data' => $rule], 201);
    }

    public function update(Request $request, int $nodeId, int $ruleId): JsonResponse
    {
        $user = Auth::user();

        $node = ServerNode::query()
            ->whereKey($nodeId)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $rule = AuditRule::query()
            ->whereKey($ruleId)
            ->where('node_id', $node->id)
            ->first();

        if (!$rule) {
            return response()->json(['success' => false, 'error' => 'Audit rule not found'], 404);
        }

        $data = $request->validate([
            'rule_type' => ['sometimes', Rule::in([AuditRule::TYPE_DOMAIN, AuditRule::TYPE_PROTOCOL, AuditRule::TYPE_IP])],
            'rule_pattern' => 'sometimes|string|max:500',
            'action' => ['sometimes', Rule::in([AuditRule::ACTION_BLOCK, AuditRule::ACTION_ALLOW, AuditRule::ACTION_LOG])],
            'is_active' => 'sometimes|boolean',
        ]);

        $rule->update($data);

        return response()->json(['success' => true, 'data' => $rule->fresh()]);
    }

    public function destroy(Request $request, int $nodeId, int $ruleId): JsonResponse
    {
        $user = Auth::user();

        $node = ServerNode::query()
            ->whereKey($nodeId)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $rule = AuditRule::query()
            ->whereKey($ruleId)
            ->where('node_id', $node->id)
            ->first();

        if (!$rule) {
            return response()->json(['success' => false, 'error' => 'Audit rule not found'], 404);
        }

        $rule->delete();

        return response()->json(['success' => true]);
    }
}

