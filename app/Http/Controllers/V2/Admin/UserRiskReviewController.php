<?php

namespace App\Http\Controllers\V2\Admin;

use App\Http\Controllers\Controller;
use App\Models\User;
use App\Models\UserRiskReview;
use App\Services\UserRiskReviewService;
use Illuminate\Http\Request;

class UserRiskReviewController extends Controller
{
    public function __construct(
        private readonly UserRiskReviewService $reviewService
    ) {
    }

    public function fetch(Request $request)
    {
        $current = max(1, (int) $request->input('current', 1));
        $pageSize = max(1, min(100, (int) $request->input('pageSize', 20)));

        $query = UserRiskReview::query()
            ->with(['user:id,email,banned,ban_reason,plan_id'])
            ->orderByDesc('reviewed_at');

        if ($request->filled('risk_level')) {
            $query->where('risk_level', (string) $request->input('risk_level'));
        }

        if ($request->filled('user_id')) {
            $query->where('user_id', (int) $request->input('user_id'));
        }

        $paginator = $query->paginate($pageSize, ['*'], 'page', $current);

        $matchedIds = collect($paginator->items())
            ->flatMap(fn (UserRiskReview $review) => collect($review->matched_user_ids ?: [])->pluck('id'))
            ->map(fn ($id) => (int) $id)
            ->unique()
            ->values();

        $matchedUsers = User::query()
            ->whereIn('id', $matchedIds->all())
            ->get(['id', 'email'])
            ->keyBy('id');

        $paginator->getCollection()->transform(function (UserRiskReview $review) use ($matchedUsers) {
            return [
                'id' => (int) $review->id,
                'user_id' => (int) $review->user_id,
                'user_email' => (string) ($review->user?->email ?? '-'),
                'user_banned' => (bool) ($review->user?->banned ?? false),
                'user_ban_reason' => $review->user?->ban_reason,
                'shared_ip' => (string) $review->shared_ip,
                'matched_user_count' => (int) $review->matched_user_count,
                'matched_users' => collect($review->matched_user_ids ?: [])
                    ->map(function ($item) use ($matchedUsers) {
                        $id = (int) data_get($item, 'id', 0);

                        return [
                            'id' => $id,
                            'email' => (string) ($matchedUsers->get($id)?->email ?? data_get($item, 'email', '-')),
                            'plan' => (string) data_get($item, 'plan', '-'),
                            'banned' => (bool) data_get($item, 'banned', false),
                        ];
                    })
                    ->values()
                    ->all(),
                'risk_level' => (string) $review->risk_level,
                'suspicion_score' => (int) $review->suspicion_score,
                'summary' => (string) ($review->summary ?? ''),
                'recommendation' => (string) ($review->recommendation ?? ''),
                'llm_model' => $review->llm_model,
                'reviewed_at' => (int) $review->reviewed_at,
                'created_at' => optional($review->created_at)->timestamp,
                'evidence' => $review->evidence,
            ];
        });

        return $this->paginate($paginator);
    }

    public function run(Request $request)
    {
        $limit = null;
        if ($request->filled('limit')) {
            $limit = max(1, min(200, (int) $request->input('limit')));
        }

        $result = $this->reviewService->scanAndReview(
            limit: $limit,
            force: true
        );

        return $this->success($result);
    }
}
