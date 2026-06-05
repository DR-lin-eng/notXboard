<?php

namespace App\Http\Controllers\V1\User;

use App\Exceptions\ApiException;
use App\Http\Controllers\Controller;
use App\Http\Resources\ComissionLogResource;
use App\Http\Resources\InviteCodeResource;
use App\Models\CommissionLog;
use App\Models\InviteCode;
use App\Models\Order;
use App\Models\Plan;
use App\Models\User;
use App\Services\Auth\InviteCodeService;
use App\Utils\Helper;
use Illuminate\Database\QueryException;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

class InviteController extends Controller
{
    public function save(Request $request)
    {
        $assignedPlanId = (int) $request->input('assigned_plan_id', 0);
        $assignedPeriod = trim((string) $request->input('assigned_period', ''));

        try {
            if ($assignedPlanId > 0 || $assignedPeriod !== '') {
                if ($assignedPlanId <= 0 || $assignedPeriod === '') {
                    throw new ApiException('邀请套餐和周期必须同时指定');
                }

                app(InviteCodeService::class)->validateAssignablePlan(
                    ownerUserId: (int) $request->user()->id,
                    planId: $assignedPlanId,
                    period: $assignedPeriod
                );
            }

            $userId = (int) $request->user()->id;
            $limit = max(1, (int) admin_setting('invite_gen_limit', 5));
            $saved = false;

            for ($attempt = 0; $attempt < 6; $attempt++) {
                try {
                    $saved = DB::transaction(function () use ($userId, $limit, $assignedPlanId, $assignedPeriod) {
                        User::query()->whereKey($userId)->lockForUpdate()->first();

                        $activeCount = InviteCode::query()
                            ->where('user_id', $userId)
                            ->where('status', InviteCode::STATUS_UNUSED)
                            ->lockForUpdate()
                            ->count();
                        if ($activeCount >= $limit) {
                            throw new ApiException(__('The maximum number of creations has been reached'));
                        }

                        $inviteCode = new InviteCode();
                        $inviteCode->user_id = $userId;
                        $inviteCode->assigned_plan_id = $assignedPlanId > 0 ? $assignedPlanId : null;
                        $inviteCode->assigned_period = $assignedPeriod !== '' ? $assignedPeriod : null;

                        for ($i = 0; $i < 8; $i++) {
                            $code = Helper::randomChar(8);
                            if (!InviteCode::query()->where('code', $code)->exists()) {
                                $inviteCode->code = $code;
                                return $inviteCode->save();
                            }
                        }

                        throw new ApiException('邀请码生成失败，请重试');
                    });
                    break;
                } catch (QueryException $e) {
                    if ($this->isDuplicateKeyException($e)) {
                        continue;
                    }
                    throw $e;
                }
            }

            if (!$saved) {
                throw new ApiException('邀请码生成失败，请重试');
            }

            return $this->success(true);
        } catch (ApiException $e) {
            return $this->fail([$e->getCode() ?: 422, $e->getMessage()]);
        } catch (\Throwable $e) {
            return $this->fail([500, __('Request failed, please try again later')]);
        }
    }

    public function details(Request $request)
    {
        $current = $request->input('current') ? $request->input('current') : 1;
        $pageSize = $request->input('page_size') >= 10 ? $request->input('page_size') : 10;
        $builder = CommissionLog::where('invite_user_id', $request->user()->id)
            ->where('get_amount', '>', 0)
            ->orderBy('created_at', 'DESC');
        $total = $builder->count();
        $details = $builder->forPage($current, $pageSize)
            ->get();
        return response([
            'data' => ComissionLogResource::collection($details),
            'total' => $total
        ]);
    }

    public function fetch(Request $request)
    {
        $commission_rate = admin_setting('invite_commission', 10);
        $user = User::find($request->user()->id)
                ->load(['codes' => fn($query) => $query->where('status', 0)->with('plan')]);
        if ($user->commission_rate) {
            $commission_rate = $user->commission_rate;
        }
        $uncheck_commission_balance = (int)Order::where('status', 3)
            ->where('commission_status', 0)
            ->where('invite_user_id', $user->id)
            ->sum('commission_balance');
        if (admin_setting('commission_distribution_enable', 0)) {
            $uncheck_commission_balance = $uncheck_commission_balance * (admin_setting('commission_distribution_l1') / 100);
        }
        $stat = [
            //已注册用户数
            (int)User::where('invite_user_id', $user->id)->count(),
            //有效的佣金
            (int)CommissionLog::where('invite_user_id', $user->id)
                ->sum('get_amount'),
            //确认中的佣金
            $uncheck_commission_balance,
            //佣金比例
            (int)$commission_rate,
            //可用佣金
            (int)$user->commission_balance
        ];
        $data = [
            'codes' => InviteCodeResource::collection($user->codes),
            'available_plans' => Plan::query()
                ->where('scope', Plan::SCOPE_NODE)
                ->where('owner_user_id', $user->id)
                ->orderByDesc('id')
                ->get()
                ->map(fn (Plan $plan) => [
                    'id' => (int) $plan->id,
                    'name' => (string) $plan->name,
                    'periods' => collect($plan->getActivePeriods())
                        ->reject(fn ($period, $key) => $key === Plan::PERIOD_RESET_TRAFFIC)
                        ->map(fn ($period, $key) => [
                            'value' => (string) $key,
                            'label' => (string) ($period['name'] ?? $key),
                        ])
                        ->values()
                        ->all(),
                ])
                ->values()
                ->all(),
            'stat' => $stat
        ];
        return $this->success($data);
    }

    private function isDuplicateKeyException(QueryException $e): bool
    {
        $sqlState = (string) ($e->errorInfo[0] ?? '');
        $driverCode = (int) ($e->errorInfo[1] ?? 0);
        if ($sqlState === '23000') {
            return true;
        }

        return $driverCode === 19;
    }
}
