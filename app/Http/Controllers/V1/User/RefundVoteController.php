<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\OrderRefundRequest;
use App\Services\OrderRefundService;
use Illuminate\Http\Request;

class RefundVoteController extends Controller
{
    public function open(Request $request)
    {
        if (!(bool) admin_setting('refund_dispute_enable', 0)) {
            return $this->fail([403, '争议退款功能已关闭']);
        }

        $items = OrderRefundRequest::query()
            ->with(['plan'])
            ->where('status', OrderRefundRequest::STATUS_VOTING)
            ->orderByDesc('id')
            ->limit(200)
            ->get();

        return $this->success($items);
    }

    public function detail(Request $request, int $id)
    {
        if (!(bool) admin_setting('refund_dispute_enable', 0)) {
            return $this->fail([403, '争议退款功能已关闭']);
        }

        $req = OrderRefundRequest::query()
            ->with(['plan', 'user', 'evidences.user', 'votes'])
            ->where('id', $id)
            ->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }
        if ((string) $req->status !== OrderRefundRequest::STATUS_VOTING) {
            // allow read even when closed (for transparency)
        }

        $approve = $req->votes->where('vote', 'approve')->count();
        $deny = $req->votes->where('vote', 'deny')->count();
        $payload = $req->toArray();
        $payload['vote_counts'] = ['approve' => $approve, 'deny' => $deny];
        $my = $req->votes->firstWhere('user_id', $request->user()->id);
        $payload['my_vote'] = $my ? (string) $my->vote : null;

        return $this->success($payload);
    }

    public function vote(Request $request, int $id)
    {
        if (!(bool) admin_setting('refund_dispute_enable', 0)) {
            return $this->fail([403, '争议退款功能已关闭']);
        }

        $data = $request->validate([
            'vote' => 'required|string|in:approve,deny',
        ]);
        $req = OrderRefundRequest::query()->where('id', $id)->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }
        app(OrderRefundService::class)->castVote($request->user(), $req, $data['vote']);
        return $this->success(true);
    }

    public function addEvidence(Request $request, int $id)
    {
        if (!(bool) admin_setting('refund_dispute_enable', 0)) {
            return $this->fail([403, '争议退款功能已关闭']);
        }

        $data = $request->validate([
            'content' => 'required|string|max:5000',
        ]);
        $req = OrderRefundRequest::query()->where('id', $id)->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }
        app(OrderRefundService::class)->addEvidence($request->user(), $req, 'member', $data['content']);
        return $this->success(true);
    }
}
