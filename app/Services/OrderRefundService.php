<?php

namespace App\Services;

use App\Exceptions\ApiException;
use App\Models\NodeTrafficRecord;
use App\Models\Order;
use App\Models\OrderRefundEvidence;
use App\Models\OrderRefundRequest;
use App\Models\OrderRefundVote;
use App\Models\Plan;
use App\Models\ServerNode;
use App\Models\User;
use App\Models\UserPlanSubscription;
use App\Services\Plugin\HookManager;
use Illuminate\Support\Carbon;
use Illuminate\Database\QueryException;
use Illuminate\Support\Facades\Crypt;
use Illuminate\Support\Facades\DB;

class OrderRefundService
{
    private const STATUS_PROCESSING = 'processing';
    private const GATEWAY_PENDING_STALE_MINUTES = 10;
    private const GATEWAY_FALLBACK_MESSAGE = 'Gateway refund failed; credited to site balance instead';

    public function createRequest(User $user, Order $order, ?string $reason = null, ?string $evidence = null): OrderRefundRequest
    {
        $plan = Plan::query()->find($order->plan_id);
        if (!$plan) {
            throw new ApiException('Plan not found');
        }
        if (($plan->scope ?? Plan::SCOPE_LEGACY) !== Plan::SCOPE_NODE) {
            throw new ApiException('Refunds are only supported for node plans currently');
        }
        if ((int) $order->user_id !== (int) $user->id) {
            throw new ApiException('Permission denied');
        }
        if ((int) $order->status !== (int) Order::STATUS_COMPLETED) {
            throw new ApiException('Only completed orders can be refunded');
        }

        [$usedKb, $allowanceKb] = $this->calculateUsageKb($order, $plan);
        [$refundCents, $chargedCents] = $this->calculateRefundCents($order, $usedKb, $allowanceKb);

        $gatewayAmount = (int) $order->total_amount + (int) ($order->handling_amount ?? 0);
        $assignedAdmin = (int) ($plan->owner_user_id ?? 0) ?: null;

        $wasCreated = false;
        try {
            $request = DB::transaction(function () use ($order, $plan, $user, $reason, $evidence, $usedKb, $allowanceKb, $refundCents, $chargedCents, $gatewayAmount, $assignedAdmin) {
                $existing = OrderRefundRequest::query()
                    ->where('order_id', $order->id)
                    ->lockForUpdate()
                    ->first();
                if ($existing) {
                    return $existing;
                }

                $req = new OrderRefundRequest();
                $req->order_id = $order->id;
                $req->trade_no = (string) $order->trade_no;
                $req->user_id = $user->id;
                $req->plan_id = $plan->id;
                $req->assigned_admin_user_id = $assignedAdmin;
                $req->status = OrderRefundRequest::STATUS_PENDING;
                $req->reason = $reason;
                $req->gateway_amount = $gatewayAmount;
                $req->gateway_trade_no = $order->callback_no;
                $req->epay_pid = $order->epay_pid ?? null;
                $req->epay_url = $order->epay_url ?? null;
                $req->epay_key_encrypted = $order->epay_key_encrypted ?? null;
                $req->used_kb = $usedKb;
                $req->allowance_kb = $allowanceKb;
                $req->refund_amount = $refundCents;
                $req->charged_amount = $chargedCents;
                if (!$req->save()) {
                    throw new ApiException('Failed to create refund request');
                }

                if ($evidence !== null && trim($evidence) !== '') {
                    OrderRefundEvidence::query()->create([
                        'refund_request_id' => $req->id,
                        'user_id' => $user->id,
                        'role' => 'user',
                        'content' => $evidence,
                    ]);
                }

                return $req;
            });
            $wasCreated = true;
        } catch (QueryException $exception) {
            $request = OrderRefundRequest::query()->where('order_id', $order->id)->first();
            if (!$request) {
                throw $exception;
            }
        }

        if ($wasCreated) {
            HookManager::call('refund.request.created', $request->fresh(['user', 'plan', 'assignedAdmin']));
        }

        return $request;
    }

    public function addEvidence(User $actor, OrderRefundRequest $request, string $role, string $content): void
    {
        $content = trim($content);
        if ($content === '') {
            throw new ApiException('Evidence content is required');
        }

        if ((int) $request->user_id !== (int) $actor->id && !$actor->is_super_admin) {
            // If assigned admin or any member, allow only when voting.
            if ((int) ($request->assigned_admin_user_id ?? 0) !== (int) $actor->id && (string) $request->status !== OrderRefundRequest::STATUS_VOTING) {
                throw new ApiException('Permission denied');
            }
        }

        OrderRefundEvidence::query()->create([
            'refund_request_id' => $request->id,
            'user_id' => $actor->id,
            'role' => $role,
            'content' => $content,
        ]);
    }

    public function adminApprove(User $admin, OrderRefundRequest $request): OrderRefundRequest
    {
        $this->assertAssignedAdminOrSuper($admin, $request);
        $request = DB::transaction(function () use ($request, $admin) {
            $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
            if (!$lockedRequest) {
                throw new ApiException('Refund request not found');
            }
            if ((string) $lockedRequest->status === OrderRefundRequest::STATUS_REFUNDED) {
                return $lockedRequest;
            }
            if (!in_array((string) $lockedRequest->status, [
                OrderRefundRequest::STATUS_PENDING,
                OrderRefundRequest::STATUS_VOTING,
                OrderRefundRequest::STATUS_APPROVED,
                OrderRefundRequest::STATUS_FAILED,
                self::STATUS_PROCESSING
            ], true)) {
                throw new ApiException('Refund request is not actionable');
            }

            $lockedRequest->status = OrderRefundRequest::STATUS_APPROVED;
            $lockedRequest->decision = 'approve';
            $lockedRequest->resolved_at = now();
            $lockedRequest->resolved_by_user_id = $admin->id;
            $lockedRequest->save();

            return $lockedRequest;
        });

        if ((string) $request->status === OrderRefundRequest::STATUS_REFUNDED) {
            return $request;
        }

        return $this->executeRefund($request, $admin);
    }

    public function adminDeny(User $admin, OrderRefundRequest $request, ?string $reason = null): OrderRefundRequest
    {
        $this->assertAssignedAdminOrSuper($admin, $request);
        $request = DB::transaction(function () use ($request, $admin, $reason) {
            $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
            if (!$lockedRequest) {
                throw new ApiException('Refund request not found');
            }
            if (!in_array((string) $lockedRequest->status, [OrderRefundRequest::STATUS_PENDING, OrderRefundRequest::STATUS_VOTING], true)) {
                throw new ApiException('Refund request is not actionable');
            }

            $lockedRequest->status = OrderRefundRequest::STATUS_DENIED;
            $lockedRequest->decision = 'deny';
            $lockedRequest->resolved_at = now();
            $lockedRequest->resolved_by_user_id = $admin->id;
            if ($reason !== null) {
                $lockedRequest->reason = $reason;
            }
            $lockedRequest->save();

            return $lockedRequest;
        });
        HookManager::call('refund.status.changed', $request->fresh(['user', 'plan', 'assignedAdmin']));
        return $request;
    }

    public function adminStartVoting(User $admin, OrderRefundRequest $request, int $minutes = 1440): OrderRefundRequest
    {
        $this->assertAssignedAdminOrSuper($admin, $request);
        $request = DB::transaction(function () use ($request, $minutes) {
            $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
            if (!$lockedRequest) {
                throw new ApiException('Refund request not found');
            }
            if ((string) $lockedRequest->status !== OrderRefundRequest::STATUS_PENDING) {
                throw new ApiException('Only pending requests can enter voting');
            }
            $lockedRequest->status = OrderRefundRequest::STATUS_VOTING;
            $lockedRequest->voting_ends_at = now()->addMinutes(max(10, $minutes));
            $lockedRequest->save();
            return $lockedRequest;
        });
        HookManager::call('refund.vote.started', $request->fresh(['user', 'plan', 'assignedAdmin']));
        return $request;
    }

    public function castVote(User $voter, OrderRefundRequest $request, string $vote): void
    {
        $vote = strtolower(trim($vote));
        if (!in_array($vote, ['approve', 'deny'], true)) {
            throw new ApiException('Invalid vote');
        }
        if ((string) $request->status !== OrderRefundRequest::STATUS_VOTING) {
            throw new ApiException('Voting is not open');
        }
        if ($request->voting_ends_at && now()->greaterThan($request->voting_ends_at)) {
            throw new ApiException('Voting has ended');
        }
        if ($voter->banned) {
            throw new ApiException('Permission denied');
        }

        OrderRefundVote::query()->updateOrCreate(
            ['refund_request_id' => $request->id, 'user_id' => $voter->id],
            ['vote' => $vote]
        );

        HookManager::call('refund.vote.cast', [
            'request' => $request->fresh(['user', 'plan', 'assignedAdmin']),
            'voter' => $voter,
            'vote' => $vote,
        ]);
    }

    public function finalizeExpiredVotings(int $limit = 200): int
    {
        $now = now();
        $requests = OrderRefundRequest::query()
            ->where('status', OrderRefundRequest::STATUS_VOTING)
            ->whereNotNull('voting_ends_at')
            ->where('voting_ends_at', '<=', $now)
            ->limit($limit)
            ->get();

        $count = 0;
        foreach ($requests as $req) {
            $this->autoResolveVoting($req);
            $count++;
        }
        return $count;
    }

    public function autoResolveVoting(OrderRefundRequest $request): OrderRefundRequest
    {
        $request = DB::transaction(function () use ($request) {
            $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
            if (!$lockedRequest) {
                throw new ApiException('Refund request not found');
            }
            if ((string) $lockedRequest->status !== OrderRefundRequest::STATUS_VOTING) {
                return $lockedRequest;
            }
            if (!$lockedRequest->voting_ends_at || now()->lessThan($lockedRequest->voting_ends_at)) {
                return $lockedRequest;
            }

            $approve = OrderRefundVote::query()->where('refund_request_id', $lockedRequest->id)->where('vote', 'approve')->count();
            $deny = OrderRefundVote::query()->where('refund_request_id', $lockedRequest->id)->where('vote', 'deny')->count();
            $decision = $approve > $deny ? 'approve' : 'deny';

            $lockedRequest->decision = $decision;
            $lockedRequest->resolved_at = now();
            $lockedRequest->resolved_by_user_id = null;
            $lockedRequest->status = $decision === 'approve'
                ? OrderRefundRequest::STATUS_APPROVED
                : OrderRefundRequest::STATUS_DENIED;
            $lockedRequest->save();

            return $lockedRequest;
        });

        if ((string) $request->status === OrderRefundRequest::STATUS_APPROVED) {
            return $this->executeRefund($request, null);
        }

        HookManager::call('refund.status.changed', $request->fresh(['user', 'plan', 'assignedAdmin']));
        return $request;
    }

    public function executeRefund(OrderRefundRequest $request, ?User $operator): OrderRefundRequest
    {
        $pendingGatewayRefundAmount = 0;
        $gatewayOrder = null;

        $request = DB::transaction(function () use ($request, $operator, &$pendingGatewayRefundAmount, &$gatewayOrder) {
            $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
            if (!$lockedRequest) {
                throw new ApiException('Refund request not found');
            }
            if ((string) $lockedRequest->status === OrderRefundRequest::STATUS_REFUNDED) {
                return $lockedRequest;
            }
            if ((string) $lockedRequest->decision !== 'approve') {
                throw new ApiException('Refund not approved');
            }
            if (!in_array((string) $lockedRequest->status, [
                OrderRefundRequest::STATUS_APPROVED,
                OrderRefundRequest::STATUS_FAILED,
                self::STATUS_PROCESSING,
            ], true)) {
                throw new ApiException('Refund request is not actionable');
            }

            $order = Order::query()->lockForUpdate()->find($lockedRequest->order_id);
            $plan = Plan::query()->find($lockedRequest->plan_id);
            $user = User::query()->lockForUpdate()->find($lockedRequest->user_id);
            if (!$order || !$plan || !$user) {
                throw new ApiException('Refund request data not found');
            }

            // Ensure usage numbers exist (may change if request created long ago).
            [$usedKb, $allowanceKb] = $this->calculateUsageKb($order, $plan);
            [$refundCents, $chargedCents] = $this->calculateRefundCents($order, $usedKb, $allowanceKb);
            $lockedRequest->used_kb = $usedKb;
            $lockedRequest->allowance_kb = $allowanceKb;
            $lockedRequest->refund_amount = $refundCents;
            $lockedRequest->charged_amount = $chargedCents;

            $balanceRefundCents = min((int) ($order->balance_amount ?? 0), $refundCents);
            $alreadyBalanceRefunded = (int) ($lockedRequest->balance_refunded_amount ?? 0);
            if ($balanceRefundCents > $alreadyBalanceRefunded) {
                $user->balance = (int) $user->balance + ($balanceRefundCents - $alreadyBalanceRefunded);
                $lockedRequest->balance_refunded_amount = $balanceRefundCents;
            }

            UserPlanSubscription::query()
                ->where('user_id', $user->id)
                ->where('order_id', $order->id)
                ->update([
                    'status' => UserPlanSubscription::STATUS_REVOKED,
                    'updated_at' => time(),
                ]);

            app(UserPlanSubscriptionService::class)->refreshUserEntitlements($user);
            if (!$user->save()) {
                throw new ApiException('Failed to recalculate user entitlements');
            }

            $lockedRequest->resolved_at = $lockedRequest->resolved_at ?? now();
            $lockedRequest->resolved_by_user_id = $lockedRequest->resolved_by_user_id ?? ($operator ? $operator->id : null);
            $lockedRequest->status = self::STATUS_PROCESSING;

            $gatewayTargetRefundCents = max(0, $refundCents - $balanceRefundCents);
            $gatewayPaidCents = (int) $order->total_amount + (int) ($order->handling_amount ?? 0);
            $gatewayTargetRefundCents = min($gatewayTargetRefundCents, $gatewayPaidCents);

            $alreadyGatewaySettled = (int) ($lockedRequest->gateway_refunded_amount ?? 0)
                + (int) ($lockedRequest->site_balance_fallback_amount ?? 0);
            $remainingGatewayRefund = max(0, $gatewayTargetRefundCents - $alreadyGatewaySettled);
            $pendingStartedAt = $lockedRequest->gateway_refund_started_at;
            $hasFreshPendingGatewayRefund = (int) ($lockedRequest->gateway_refund_pending_amount ?? 0) > 0
                && $pendingStartedAt
                && $pendingStartedAt->greaterThan(now()->subMinutes(self::GATEWAY_PENDING_STALE_MINUTES));

            if ($remainingGatewayRefund <= 0) {
                $this->finalizeRefundRequest($lockedRequest, $operator);
                $lockedRequest->save();
                return $lockedRequest;
            }

            if ($hasFreshPendingGatewayRefund) {
                $lockedRequest->save();
                return $lockedRequest;
            }

            $lockedRequest->gateway_refund_pending_amount = $remainingGatewayRefund;
            $lockedRequest->gateway_refund_started_at = now();
            $lockedRequest->save();

            $pendingGatewayRefundAmount = $remainingGatewayRefund;
            $gatewayOrder = $order;
            return $lockedRequest;
        });

        if ($pendingGatewayRefundAmount <= 0) {
            HookManager::call('refund.status.changed', $request->fresh(['user', 'plan', 'assignedAdmin']));
            return $request;
        }

        try {
            if ($this->tryGatewayRefund($gatewayOrder, $request, $pendingGatewayRefundAmount)) {
                $request = DB::transaction(function () use ($request, $operator) {
                    $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
                    if (!$lockedRequest) {
                        throw new ApiException('Refund request not found');
                    }

                    $pendingAmount = (int) ($lockedRequest->gateway_refund_pending_amount ?? 0);
                    if ($pendingAmount > 0) {
                        $lockedRequest->gateway_refunded_amount = (int) ($lockedRequest->gateway_refunded_amount ?? 0) + $pendingAmount;
                    }
                    $lockedRequest->gateway_refund_pending_amount = 0;
                    $lockedRequest->gateway_refund_started_at = null;
                    $this->finalizeRefundRequest($lockedRequest, $operator);
                    $lockedRequest->save();

                    return $lockedRequest;
                });

                HookManager::call('refund.status.changed', $request->fresh(['user', 'plan', 'assignedAdmin']));
                return $request;
            }
        } catch (\Throwable $e) {
            $request = $this->markRefundAsFailed($request, $operator, $e->getMessage());
            HookManager::call('refund.status.changed', $request->fresh(['user', 'plan', 'assignedAdmin']));
            throw $e;
        }

        try {
            $request = DB::transaction(function () use ($request, $operator) {
                $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
                if (!$lockedRequest) {
                    throw new ApiException('Refund request not found');
                }

                $pendingAmount = (int) ($lockedRequest->gateway_refund_pending_amount ?? 0);
                if ($pendingAmount > 0) {
                    $user = User::query()->lockForUpdate()->find($lockedRequest->user_id);
                    if (!$user) {
                        throw new ApiException('Refund request data not found');
                    }

                    $user->balance = (int) $user->balance + $pendingAmount;
                    if (!$user->save()) {
                        throw new ApiException('Failed to refund balance');
                    }

                    $lockedRequest->site_balance_fallback_amount = (int) ($lockedRequest->site_balance_fallback_amount ?? 0) + $pendingAmount;
                }

                $lockedRequest->gateway_refund_pending_amount = 0;
                $lockedRequest->gateway_refund_started_at = null;
                $lockedRequest->reason = $this->appendReasonLine($lockedRequest->reason, self::GATEWAY_FALLBACK_MESSAGE);
                $this->finalizeRefundRequest($lockedRequest, $operator);
                $lockedRequest->save();

                return $lockedRequest;
            });
        } catch (\Throwable $e) {
            $request = $this->markRefundAsFailed($request, $operator, $e->getMessage());
            HookManager::call('refund.status.changed', $request->fresh(['user', 'plan', 'assignedAdmin']));
            throw $e;
        }

        HookManager::call('refund.status.changed', $request->fresh(['user', 'plan', 'assignedAdmin']));
        return $request;
    }

    protected function tryGatewayRefund(Order $order, OrderRefundRequest $request, int $refundCents): bool
    {
        if (!$order->callback_no) {
            return false;
        }
        $pid = (string) ($order->epay_pid ?? ($request->epay_pid ?? ''));
        $url = (string) ($order->epay_url ?? ($request->epay_url ?? ''));
        $keyEncrypted = (string) ($order->epay_key_encrypted ?? ($request->epay_key_encrypted ?? ''));
        if ($pid === '' || $url === '' || $keyEncrypted === '') {
            return false;
        }

        $key = Crypt::decryptString($keyEncrypted);
        $money = number_format(((float) $refundCents) / 100, 2, '.', '');

        $res = app(EpayApiService::class)->refund(
            $url,
            $pid,
            $key,
            (string) $order->callback_no,
            (string) $money,
            (string) $order->trade_no
        );
        return (bool) ($res['ok'] ?? false);
    }

    protected function calculateUsageKb(Order $order, Plan $plan): array
    {
        $nodeIds = collect($plan->node_ids ?? [])
            ->map(fn ($v) => (int) $v)
            ->filter(fn ($v) => $v > 0)
            ->unique()
            ->values()
            ->all();

        $validNodeIds = [];
        if (!empty($nodeIds)) {
            $validNodeIds = ServerNode::query()->whereIn('id', $nodeIds)->pluck('id')->map(fn ($v) => (int) $v)->all();
        }

        $start = Carbon::createFromTimestamp((int) ($order->paid_at ?: $order->created_at))->startOfDay();
        $end = now()->endOfDay();

        $usedKb = 0;
        if (!empty($validNodeIds)) {
            $usedKb = (int) NodeTrafficRecord::query()
                ->where('user_id', $order->user_id)
                ->whereIn('node_id', $validNodeIds)
                ->whereBetween('record_date', [$start->toDateString(), $end->toDateString()])
                ->sum(DB::raw('upload_traffic + download_traffic'));
        }

        // Plan.transfer_enable is treated as GB in legacy code. Convert to KB.
        $allowanceKb = max(0, (int) $plan->transfer_enable) * 1024 * 1024;

        return [$usedKb, $allowanceKb];
    }

    protected function calculateRefundCents(Order $order, int $usedKb, int $allowanceKb): array
    {
        $paidCents = (int) $order->total_amount + (int) ($order->handling_amount ?? 0) + (int) ($order->balance_amount ?? 0);
        if ($paidCents <= 0) {
            return [0, 0];
        }

        if ($allowanceKb <= 0) {
            return [$paidCents, 0];
        }

        $ratio = min(1.0, max(0.0, $usedKb / $allowanceKb));
        $charged = (int) round($paidCents * $ratio);
        $charged = max(0, min($paidCents, $charged));
        $refund = $paidCents - $charged;
        return [$refund, $charged];
    }

    protected function assertAssignedAdminOrSuper(User $admin, OrderRefundRequest $request): void
    {
        if ($admin->is_super_admin) {
            return;
        }
        if ((int) ($request->assigned_admin_user_id ?? 0) !== (int) $admin->id) {
            throw new ApiException('Permission denied');
        }
    }

    private function finalizeRefundRequest(OrderRefundRequest $request, ?User $operator): void
    {
        $targetRefundAmount = max(0, (int) ($request->refund_amount ?? 0));
        $settledRefundAmount = (int) ($request->balance_refunded_amount ?? 0)
            + (int) ($request->gateway_refunded_amount ?? 0)
            + (int) ($request->site_balance_fallback_amount ?? 0);

        if ($settledRefundAmount >= $targetRefundAmount) {
            $request->status = OrderRefundRequest::STATUS_REFUNDED;
            $request->refunded_at = $request->refunded_at ?? now();
        } else {
            $request->status = self::STATUS_PROCESSING;
        }

        $request->resolved_at = $request->resolved_at ?? now();
        $request->resolved_by_user_id = $request->resolved_by_user_id ?? ($operator ? $operator->id : null);
    }

    private function markRefundAsFailed(OrderRefundRequest $request, ?User $operator, string $error): OrderRefundRequest
    {
        return DB::transaction(function () use ($request, $operator, $error) {
            $lockedRequest = OrderRefundRequest::query()->lockForUpdate()->find($request->id);
            if (!$lockedRequest) {
                throw new ApiException('Refund request not found');
            }

            $lockedRequest->status = OrderRefundRequest::STATUS_FAILED;
            $lockedRequest->gateway_refund_pending_amount = 0;
            $lockedRequest->gateway_refund_started_at = null;
            $lockedRequest->resolved_at = $lockedRequest->resolved_at ?? now();
            $lockedRequest->resolved_by_user_id = $lockedRequest->resolved_by_user_id ?? ($operator ? $operator->id : null);
            $lockedRequest->reason = $this->appendReasonLine($lockedRequest->reason, 'Refund processing failed: ' . $error);
            $lockedRequest->save();

            return $lockedRequest;
        });
    }

    private function appendReasonLine(?string $reason, string $line): string
    {
        $normalizedReason = trim((string) $reason);
        $normalizedLine = trim($line);

        if ($normalizedLine === '') {
            return $normalizedReason;
        }

        if ($normalizedReason === '') {
            return $normalizedLine;
        }

        if (str_contains($normalizedReason, $normalizedLine)) {
            return $normalizedReason;
        }

        return $normalizedReason . "\n" . $normalizedLine;
    }
}
