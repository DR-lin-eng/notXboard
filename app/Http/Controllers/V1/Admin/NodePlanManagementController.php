<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Http\Resources\PlanResource;
use App\Models\Plan;
use App\Models\ServerNode;
use App\Services\PlanService;
use Illuminate\Http\Request;
use Illuminate\Validation\Rule;

class NodePlanManagementController extends Controller
{
    private function normalizePrices(array $prices): array
    {
        $normalized = [];
        foreach ($prices as $key => $value) {
            if (!is_string($key)) {
                continue;
            }
            $periodKey = PlanService::getPeriodKey($key);
            if (!Plan::isValidPeriod($periodKey)) {
                continue;
            }
            $numericPrice = is_numeric($value) ? round((float) $value, 2) : null;
            if ($numericPrice === null || $numericPrice < 0) {
                continue;
            }
            $normalized[$periodKey] = $numericPrice;
        }
        return $normalized;
    }

    private function normalizeAccessUserIds(mixed $raw): array
    {
        if (!is_array($raw)) {
            return [];
        }

        return collect($raw)
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->unique()
            ->values()
            ->all();
    }

    private function normalizeShareToken(mixed $raw): ?string
    {
        $token = trim((string) ($raw ?? ''));
        return $token === '' ? null : $token;
    }

    private function normalizeFreeQuota(mixed $raw): ?array
    {
        if (!is_array($raw)) {
            return null;
        }

        $result = [];
        $hasPositive = false;
        foreach ($raw as $level => $value) {
            $intLevel = (int) $level;
            if ($intLevel < 0 || $intLevel > 4) {
                continue;
            }
            $numeric = is_numeric($value) ? round((float) $value, 2) : 0;
            $normalized = $numeric > 0 ? $numeric : 0;
            $result[(string) $intLevel] = $normalized;
            if ($normalized > 0) {
                $hasPositive = true;
            }
        }

        return $hasPositive ? $result : null;
    }

    public function index(Request $request)
    {
        $plans = Plan::query()
            ->where('scope', Plan::SCOPE_NODE)
            ->orderByDesc('id')
            ->get();

        return $this->success(PlanResource::collection($plans));
    }

    public function nodeOptions(Request $request)
    {
        $nodes = ServerNode::query()
            ->with(['owner:id,email'])
            ->orderByDesc('id')
            ->get(['id', 'name', 'user_id', 'status']);

        $data = $nodes->map(function (ServerNode $node) {
            return [
                'id' => (int) $node->id,
                'name' => (string) $node->name,
                'user_id' => (int) $node->user_id,
                'owner_email' => $node->owner?->email,
                'status' => (string) $node->status,
                'online_status' => (string) $node->getOnlineStatus(),
                'is_online' => (bool) $node->isReportedOnline(),
                'last_report_at' => $node->getLastReportAt(),
            ];
        })->values();

        return $this->success($data);
    }

    public function create(Request $request)
    {
        $data = $request->validate([
            'name' => 'required|string|max:255',
            'content' => 'nullable|string',
            'prices' => 'required|array',
            'prices.*' => 'numeric|min:0',
            'sell' => 'required|boolean',
            'show' => 'required|boolean',
            'renew' => 'required|boolean',
            'sort' => 'nullable|integer',
            'capacity_limit' => 'nullable|integer',
            'device_limit' => 'nullable|integer',
            'speed_limit' => 'nullable|integer',
            'transfer_enable' => 'nullable|integer',
            'node_ids' => 'required|array|min:1',
            'node_ids.*' => 'integer|min:1',
            'owner_user_id' => 'nullable|integer|min:1|exists:v2_user,id',
            'min_trust_level' => 'nullable|integer|min:0|max:4',
            'allow_trial' => 'nullable|boolean',
            'is_unlimited_traffic' => 'nullable|boolean',
            'free_quota_gb_by_trust_level' => 'nullable|array',
            'free_quota_gb_by_trust_level.*' => 'numeric|min:0',
            'visibility_scope' => ['nullable', 'string', Rule::in([
                Plan::VISIBILITY_PUBLIC,
                Plan::VISIBILITY_LINK_ONLY,
                Plan::VISIBILITY_ASSIGNED_ONLY,
            ])],
            'access_user_ids' => 'nullable|array',
            'access_user_ids.*' => 'integer|min:1|exists:v2_user,id',
            'share_token' => 'nullable|string|min:8|max:64|regex:/^[A-Za-z0-9_-]+$/|unique:v2_plan,share_token',
        ]);

        $nodeIds = collect($data['node_ids'])->map(fn ($v) => (int) $v)->unique()->values()->all();
        $validNodeIds = ServerNode::query()
            ->whereIn('id', $nodeIds)
            ->pluck('id')
            ->map(fn ($v) => (int) $v)
            ->all();
        if (count($validNodeIds) !== count($nodeIds)) {
            return response()->json(['success' => false, 'error' => 'node_ids contains invalid nodes'], 422);
        }

        $prices = $this->normalizePrices($data['prices']);
        if (empty($prices)) {
            return response()->json(['success' => false, 'error' => 'prices is empty or invalid'], 422);
        }

        $visibilityScope = Plan::normalizeVisibilityScope($data['visibility_scope'] ?? Plan::VISIBILITY_PUBLIC);
        $accessUserIds = $visibilityScope === Plan::VISIBILITY_ASSIGNED_ONLY
            ? $this->normalizeAccessUserIds($data['access_user_ids'] ?? [])
            : [];
        if ($visibilityScope === Plan::VISIBILITY_ASSIGNED_ONLY && empty($accessUserIds)) {
            return response()->json(['success' => false, 'error' => 'assigned users cannot be empty'], 422);
        }

        $allowTrial = array_key_exists('allow_trial', $data)
            ? (bool) $data['allow_trial']
            : true;
        $freeQuota = $allowTrial
            ? $this->normalizeFreeQuota($data['free_quota_gb_by_trust_level'] ?? null)
            : null;
        $shareToken = $this->normalizeShareToken($data['share_token'] ?? null);

        $plan = Plan::create([
            'scope' => Plan::SCOPE_NODE,
            'owner_user_id' => $data['owner_user_id'] ?? null,
            'min_trust_level' => $data['min_trust_level'] ?? null,
            'free_quota_gb_by_trust_level' => $freeQuota,
            'node_ids' => $nodeIds,
            'group_id' => null,
            'transfer_enable' => (int) ($data['transfer_enable'] ?? 0),
            'is_unlimited_traffic' => (bool) ($data['is_unlimited_traffic'] ?? false),
            'name' => $data['name'],
            'speed_limit' => $data['speed_limit'] ?? null,
            'show' => $visibilityScope === Plan::VISIBILITY_PUBLIC ? (bool) $data['show'] : false,
            'visibility_scope' => $visibilityScope,
            'access_user_ids' => $accessUserIds,
            'share_token' => $shareToken,
            'sort' => (int) ($data['sort'] ?? 0),
            'renew' => (bool) $data['renew'],
            'content' => $data['content'] ?? null,
            'prices' => $prices,
            'capacity_limit' => $data['capacity_limit'] ?? null,
            'sell' => (bool) $data['sell'],
            'device_limit' => $data['device_limit'] ?? null,
            'tags' => [],
        ]);

        if ($visibilityScope === Plan::VISIBILITY_LINK_ONLY && !$shareToken) {
            $plan->ensureShareToken();
            $plan->save();
        }

        return $this->success(PlanResource::make($plan));
    }

    public function update(Request $request, int $id)
    {
        $plan = Plan::query()
            ->whereKey($id)
            ->where('scope', Plan::SCOPE_NODE)
            ->first();

        if (!$plan) {
            return $this->fail([400, __('Subscription plan does not exist')]);
        }

        $data = $request->validate([
            'name' => 'sometimes|string|max:255',
            'content' => 'sometimes|nullable|string',
            'prices' => 'sometimes|array',
            'prices.*' => 'numeric|min:0',
            'sell' => 'sometimes|boolean',
            'show' => 'sometimes|boolean',
            'renew' => 'sometimes|boolean',
            'sort' => 'sometimes|integer',
            'capacity_limit' => 'sometimes|nullable|integer',
            'device_limit' => 'sometimes|nullable|integer',
            'speed_limit' => 'sometimes|nullable|integer',
            'transfer_enable' => 'sometimes|integer',
            'node_ids' => 'sometimes|array|min:1',
            'node_ids.*' => 'integer|min:1',
            'owner_user_id' => 'sometimes|nullable|integer|min:1|exists:v2_user,id',
            'min_trust_level' => 'sometimes|nullable|integer|min:0|max:4',
            'allow_trial' => 'sometimes|boolean',
            'is_unlimited_traffic' => 'sometimes|boolean',
            'free_quota_gb_by_trust_level' => 'sometimes|nullable|array',
            'free_quota_gb_by_trust_level.*' => 'numeric|min:0',
            'visibility_scope' => ['sometimes', 'nullable', 'string', Rule::in([
                Plan::VISIBILITY_PUBLIC,
                Plan::VISIBILITY_LINK_ONLY,
                Plan::VISIBILITY_ASSIGNED_ONLY,
            ])],
            'access_user_ids' => 'sometimes|nullable|array',
            'access_user_ids.*' => 'integer|min:1|exists:v2_user,id',
            'share_token' => [
                'sometimes',
                'nullable',
                'string',
                'min:8',
                'max:64',
                'regex:/^[A-Za-z0-9_-]+$/',
                Rule::unique('v2_plan', 'share_token')->ignore($plan->id),
            ],
        ]);

        if (isset($data['prices'])) {
            $prices = $this->normalizePrices($data['prices']);
            if (empty($prices)) {
                return response()->json(['success' => false, 'error' => 'prices is empty or invalid'], 422);
            }
            $data['prices'] = $prices;
        }

        if (isset($data['node_ids'])) {
            $nodeIds = collect($data['node_ids'])->map(fn ($v) => (int) $v)->unique()->values()->all();
            $validNodeIds = ServerNode::query()
                ->whereIn('id', $nodeIds)
                ->pluck('id')
                ->map(fn ($v) => (int) $v)
                ->all();
            if (count($validNodeIds) !== count($nodeIds)) {
                return response()->json(['success' => false, 'error' => 'node_ids contains invalid nodes'], 422);
            }
            $data['node_ids'] = $nodeIds;
        }

        if (array_key_exists('visibility_scope', $data)) {
            $data['visibility_scope'] = Plan::normalizeVisibilityScope($data['visibility_scope']);
        }

        $effectiveScope = $data['visibility_scope'] ?? Plan::normalizeVisibilityScope($plan->visibility_scope ?? null);
        if ($effectiveScope === Plan::VISIBILITY_ASSIGNED_ONLY) {
            $sourceIds = array_key_exists('access_user_ids', $data) ? ($data['access_user_ids'] ?? []) : ($plan->access_user_ids ?? []);
            $normalizedIds = $this->normalizeAccessUserIds($sourceIds);
            if (empty($normalizedIds)) {
                return response()->json(['success' => false, 'error' => 'assigned users cannot be empty'], 422);
            }
            $data['access_user_ids'] = $normalizedIds;
        } else {
            $data['access_user_ids'] = [];
        }

        if ($effectiveScope !== Plan::VISIBILITY_PUBLIC) {
            $data['show'] = false;
        }

        if (array_key_exists('share_token', $data)) {
            $data['share_token'] = $this->normalizeShareToken($data['share_token']);
        }

        if (array_key_exists('free_quota_gb_by_trust_level', $data)) {
            $data['free_quota_gb_by_trust_level'] = $this->normalizeFreeQuota($data['free_quota_gb_by_trust_level']);
        }
        if (array_key_exists('allow_trial', $data) && !$data['allow_trial']) {
            $data['free_quota_gb_by_trust_level'] = null;
        }
        if (array_key_exists('is_unlimited_traffic', $data)) {
            $data['is_unlimited_traffic'] = (bool) $data['is_unlimited_traffic'];
        }
        unset($data['allow_trial']);

        $plan->update($data);

        if ($effectiveScope === Plan::VISIBILITY_LINK_ONLY && empty($plan->share_token)) {
            $plan->ensureShareToken();
            $plan->save();
        }

        return $this->success(PlanResource::make($plan->fresh()));
    }

    public function delete(Request $request, int $id)
    {
        $plan = Plan::query()
            ->whereKey($id)
            ->where('scope', Plan::SCOPE_NODE)
            ->first();

        if (!$plan) {
            return $this->fail([400, __('Subscription plan does not exist')]);
        }

        $plan->delete();
        return $this->success(true);
    }
}
