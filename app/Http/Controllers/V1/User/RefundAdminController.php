<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\OrderRefundRequest;
use App\Services\OrderRefundService;
use Illuminate\Http\Request;

class RefundAdminController extends Controller
{
    public function inbox(Request $request)
    {
        $items = OrderRefundRequest::query()
            ->with(['plan', 'user'])
            ->where('assigned_admin_user_id', $request->user()->id)
            ->orderByDesc('id')
            ->limit(200)
            ->get();

        return $this->success($items);
    }

    public function detail(Request $request, int $id)
    {
        $req = OrderRefundRequest::query()
            ->with(['plan', 'user', 'evidences.user', 'votes'])
            ->where('id', $id)
            ->where('assigned_admin_user_id', $request->user()->id)
            ->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }

        $approve = $req->votes->where('vote', 'approve')->count();
        $deny = $req->votes->where('vote', 'deny')->count();
        $payload = $req->toArray();
        $payload['vote_counts'] = ['approve' => $approve, 'deny' => $deny];

        return $this->success($payload);
    }

    public function approve(Request $request, int $id)
    {
        $req = OrderRefundRequest::query()
            ->where('id', $id)
            ->where('assigned_admin_user_id', $request->user()->id)
            ->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }
        $out = app(OrderRefundService::class)->adminApprove($request->user(), $req);
        return $this->success($out);
    }

    public function deny(Request $request, int $id)
    {
        $data = $request->validate([
            'reason' => 'nullable|string|max:2000',
        ]);
        $req = OrderRefundRequest::query()
            ->where('id', $id)
            ->where('assigned_admin_user_id', $request->user()->id)
            ->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }
        $out = app(OrderRefundService::class)->adminDeny($request->user(), $req, $data['reason'] ?? null);
        return $this->success($out);
    }

    public function dispute(Request $request, int $id)
    {
        if (!(bool) admin_setting('refund_dispute_enable', 0)) {
            return $this->fail([403, '争议退款功能已关闭']);
        }

        $data = $request->validate([
            'minutes' => 'nullable|integer|min:10|max:10080',
            'evidence' => 'nullable|string|max:5000',
        ]);
        $req = OrderRefundRequest::query()
            ->where('id', $id)
            ->where('assigned_admin_user_id', $request->user()->id)
            ->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }

        if (!empty($data['evidence'])) {
            app(OrderRefundService::class)->addEvidence($request->user(), $req, 'admin', $data['evidence']);
        }

        $out = app(OrderRefundService::class)->adminStartVoting($request->user(), $req, (int) ($data['minutes'] ?? 1440));
        return $this->success($out);
    }

    public function addEvidence(Request $request, int $id)
    {
        $data = $request->validate([
            'content' => 'required|string|max:5000',
        ]);
        $req = OrderRefundRequest::query()
            ->where('id', $id)
            ->where('assigned_admin_user_id', $request->user()->id)
            ->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }

        app(OrderRefundService::class)->addEvidence($request->user(), $req, 'admin', $data['content']);
        return $this->success(true);
    }
}
