<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\Payment;
use App\Models\SponsorDonation;
use App\Services\PaymentProfileService;
use App\Services\PaymentService;
use App\Utils\Helper;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class SponsorController extends Controller
{
    public function create(Request $request): JsonResponse
    {
        $data = $request->validate([
            'amount' => 'required|numeric|min:0.01|max:100000',
        ]);

        $user = Auth::user();
        $totalAmount = (int) round(((float) $data['amount']) * 100);

        $donation = SponsorDonation::create([
            'user_id' => $user?->id,
            'trade_no' => Helper::generateOrderNo(),
            'total_amount' => $totalAmount,
            'status' => SponsorDonation::STATUS_PENDING,
        ]);

        return response()->json([
            'success' => true,
            'data' => [
                'trade_no' => $donation->trade_no,
                'total_amount' => $donation->total_amount,
                'status' => $donation->status,
            ],
        ]);
    }

    public function detail(Request $request, string $tradeNo): JsonResponse
    {
        $user = Auth::user();
        $donation = SponsorDonation::query()
            ->where('trade_no', $tradeNo)
            ->when($user?->id, fn ($q) => $q->where('user_id', $user->id))
            ->first();

        if (!$donation) {
            return response()->json(['success' => false, 'error' => 'Donation not found'], 404);
        }

        return response()->json(['success' => true, 'data' => $donation]);
    }

    public function methods(Request $request): JsonResponse
    {
        $methods = Payment::select(['id', 'name', 'payment', 'icon'])
            ->where('enable', 1)
            ->orderBy('sort', 'ASC')
            ->get()
            ->filter(fn ($p) => (string) $p->payment === 'EPay')
            ->values();

        return response()->json(['success' => true, 'data' => $methods]);
    }

    public function checkout(Request $request): JsonResponse
    {
        $data = $request->validate([
            'trade_no' => 'required|string',
            'method' => 'required|integer',
        ]);

        $user = Auth::user();
        $donation = SponsorDonation::query()
            ->where('trade_no', $data['trade_no'])
            ->when($user?->id, fn ($q) => $q->where('user_id', $user->id))
            ->where('status', SponsorDonation::STATUS_PENDING)
            ->first();

        if (!$donation) {
            return response()->json(['success' => false, 'error' => 'Donation not found or already paid'], 404);
        }

        $payment = Payment::find($data['method']);
        if (!$payment || !$payment->enable || (string) $payment->payment !== 'EPay') {
            return response()->json(['success' => false, 'error' => 'Payment method not available'], 400);
        }

        $sponsor = app(PaymentProfileService::class)->getSponsorEpayProfile();
        if (!$sponsor) {
            return response()->json(['success' => false, 'error' => 'Sponsor payment profile not configured'], 400);
        }

        $donation->payment_id = $payment->id;
        $donation->save();

        $overrideConfig = [
            'pid' => $sponsor['pid'],
            'key' => $sponsor['key'],
            'url' => $sponsor['url'],
            'submit_path' => $sponsor['submit_path'] ?? '/pay/submit.php',
            'use_post' => $sponsor['use_post'] ?? true,
            'sitename' => $sponsor['sitename'] ?? null,
            'device' => $sponsor['device'] ?? null,
        ];

        $paymentService = new PaymentService($payment->payment, $payment->id, null, $overrideConfig);
        $result = $paymentService->pay([
            'trade_no' => $donation->trade_no,
            'total_amount' => $donation->total_amount,
            'user_id' => $donation->user_id,
            'stripe_token' => null,
            'return_url' => source_base_url('/#/sponsor/' . $donation->trade_no),
        ]);

        return response()->json([
            'success' => true,
            'type' => $result['type'],
            'data' => $result['data'],
        ]);
    }
}

