<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\Notice;
use App\Models\Plan;
use App\Models\User;
use App\Models\UserPlanSubscription;
use App\Services\Plugin\HookManager;
use Illuminate\Http\Request;

class NoticeController extends Controller
{
    public function fetch(Request $request)
    {
        /** @var User $user */
        $user = $request->user();
        $current = $request->input('current') ? $request->input('current') : 1;
        $pageSize = max(5, min(20, (int) $request->input('page_size', 10)));
        $activePlanIds = UserPlanSubscription::query()
            ->where('user_id', $user->id)
            ->active(time())
            ->pluck('plan_id')
            ->map(fn ($value) => (int) $value)
            ->unique()
            ->values()
            ->all();

        $visible = Notice::query()
            ->orderBy('sort', 'ASC')
            ->orderBy('id', 'DESC')
            ->where('show', true)
            ->get()
            ->filter(function (Notice $notice) use ($user, $activePlanIds) {
                if ((string) ($notice->scope_type ?? Notice::SCOPE_GLOBAL) === Notice::SCOPE_GLOBAL) {
                    return true;
                }

                if ((int) ($notice->author_user_id ?? 0) === (int) $user->id) {
                    return true;
                }

                $targetPlanIds = collect($notice->target_plan_ids ?? [])
                    ->map(fn ($value) => (int) $value)
                    ->filter(fn ($value) => $value > 0)
                    ->all();

                return !empty(array_intersect($activePlanIds, $targetPlanIds));
            })
            ->values();

        $total = $visible->count();
        $res = $visible->forPage($current, $pageSize)->values();

        $planOptions = Plan::query()
            ->where('scope', Plan::SCOPE_NODE)
            ->where('owner_user_id', $user->id)
            ->orderByDesc('id')
            ->get(['id', 'name'])
            ->map(fn (Plan $plan) => [
                'id' => (int) $plan->id,
                'name' => (string) $plan->name,
            ])
            ->values()
            ->all();

        $manageable = Notice::query()
            ->where('author_user_id', $user->id)
            ->where('scope_type', Notice::SCOPE_PLAN_SUBSCRIBERS)
            ->orderBy('id', 'DESC')
            ->limit(50)
            ->get()
            ->map(function (Notice $notice) {
                return [
                    'id' => (int) $notice->id,
                    'title' => (string) $notice->title,
                    'content' => (string) $notice->content,
                    'tags' => $notice->tags ?? [],
                    'show' => (bool) $notice->show,
                    'target_plan_ids' => collect($notice->target_plan_ids ?? [])->map(fn ($v) => (int) $v)->values()->all(),
                    'created_at' => $notice->created_at,
                    'updated_at' => $notice->updated_at,
                ];
            })
            ->values()
            ->all();

        return response([
            'data' => $res,
            'total' => $total,
            'can_publish' => !empty($planOptions),
            'plan_options' => $planOptions,
            'manageable' => $manageable,
        ]);
    }

    public function save(Request $request)
    {
        /** @var User $user */
        $user = $request->user();

        $data = $request->validate([
            'title' => 'required|string|max:255',
            'content' => 'required|string',
            'tags' => 'nullable|array',
            'tags.*' => 'string|max:32',
            'target_plan_ids' => 'required|array|min:1',
            'target_plan_ids.*' => 'integer|min:1',
        ]);

        $planIds = collect($data['target_plan_ids'])
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->unique()
            ->values()
            ->all();

        $ownedPlanIds = Plan::query()
            ->where('scope', Plan::SCOPE_NODE)
            ->where('owner_user_id', $user->id)
            ->whereIn('id', $planIds)
            ->pluck('id')
            ->map(fn ($value) => (int) $value)
            ->all();

        if (count($ownedPlanIds) !== count($planIds)) {
            return response()->json(['success' => false, 'error' => 'target_plan_ids contains plans you do not own'], 403);
        }

        $notice = Notice::query()->create([
            'title' => $data['title'],
            'content' => $data['content'],
            'tags' => $data['tags'] ?? [],
            'show' => true,
            'popup' => false,
            'scope_type' => Notice::SCOPE_PLAN_SUBSCRIBERS,
            'target_plan_ids' => $ownedPlanIds,
            'author_user_id' => $user->id,
        ]);

        HookManager::call('notice.published', [
            'notice' => $notice->fresh(),
            'source' => 'user',
            'author_user_id' => $user->id,
        ]);

        return response()->json([
            'success' => true,
            'data' => $notice,
        ]);
    }

    public function toggle(Request $request, int $id)
    {
        $notice = Notice::query()
            ->whereKey($id)
            ->where('author_user_id', $request->user()->id)
            ->where('scope_type', Notice::SCOPE_PLAN_SUBSCRIBERS)
            ->first();

        if (!$notice) {
            return response()->json(['success' => false, 'error' => 'Notice not found'], 404);
        }

        $notice->show = !$notice->show;
        $notice->save();

        if ((bool) $notice->show) {
            HookManager::call('notice.published', [
                'notice' => $notice->fresh(),
                'source' => 'user',
                'author_user_id' => $request->user()->id,
            ]);
        }

        return response()->json(['success' => true, 'data' => true]);
    }

    public function drop(Request $request, int $id)
    {
        $notice = Notice::query()
            ->whereKey($id)
            ->where('author_user_id', $request->user()->id)
            ->where('scope_type', Notice::SCOPE_PLAN_SUBSCRIBERS)
            ->first();

        if (!$notice) {
            return response()->json(['success' => false, 'error' => 'Notice not found'], 404);
        }

        $notice->delete();
        return response()->json(['success' => true, 'data' => true]);
    }
}
