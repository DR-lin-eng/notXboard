<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Models\OrderRefundRequest;
use App\Services\OrderRefundService;
use Illuminate\Http\Request;

class RefundManagementController extends Controller
{
    public function index(Request $request)
    {
        $items = OrderRefundRequest::query()
            ->with(['plan', 'user'])
            ->orderByDesc('id')
            ->limit(500)
            ->get();
        return $this->success($items);
    }

    public function detail(Request $request, int $id)
    {
        $req = OrderRefundRequest::query()
            ->with(['plan', 'user', 'evidences.user', 'votes'])
            ->where('id', $id)
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
        $req = OrderRefundRequest::query()->where('id', $id)->first();
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
        $req = OrderRefundRequest::query()->where('id', $id)->first();
        if (!$req) {
            return $this->fail([404, 'Refund request not found']);
        }
        $out = app(OrderRefundService::class)->adminDeny($request->user(), $req, $data['reason'] ?? null);
        return $this->success($out);
    }

    public function finalize(Request $request)
    {
        $count = app(OrderRefundService::class)->finalizeExpiredVotings(500);
        return $this->success(['processed' => $count]);
    }
}

