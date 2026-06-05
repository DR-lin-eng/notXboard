<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\Order;
use App\Models\OrderRefundRequest;
use App\Services\OrderRefundService;
use Illuminate\Http\Request;

class RefundController extends Controller
{
    public function myRequests(Request $request)
    {
        $items = OrderRefundRequest::query()
            ->with(['plan'])
            ->where('user_id', $request->user()->id)
            ->orderByDesc('id')
            ->limit(200)
            ->get();

        return $this->success($items);
    }

    public function create(Request $request)
    {
        $data = $request->validate([
            'trade_no' => 'required|string',
            'reason' => 'nullable|string|max:2000',
            'evidence' => 'nullable|string|max:5000',
        ]);

        $order = Order::query()
            ->where('trade_no', $data['trade_no'])
            ->where('user_id', $request->user()->id)
            ->first();

        if (!$order) {
            return $this->fail([400, 'Order not found']);
        }

        $req = app(OrderRefundService::class)->createRequest($request->user(), $order, $data['reason'] ?? null, $data['evidence'] ?? null);
        return $this->success($req);
    }

    public function detail(Request $request, int $id)
    {
        $req = OrderRefundRequest::query()
            ->with(['plan', 'evidences.user', 'votes'])
            ->where('id', $id)
            ->where('user_id', $request->user()->id)
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

    public function addEvidence(Request $request, int $id)
    {
        $data = $request->validate([
            'content' => 'required|string|max:5000',
        ]);

        $req = OrderRefundRequest::query()
            ->where('id', $id)
            ->where('user_id', $request->user()->id)
            ->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }

        app(OrderRefundService::class)->addEvidence($request->user(), $req, 'user', $data['content']);
        return $this->success(true);
    }
}

