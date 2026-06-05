<?php

namespace App\Http\Controllers\V1\Guest;

use App\Http\Controllers\Controller;
use App\Models\Order;
use App\Models\SponsorDonation;
use App\Services\OrderService;
use App\Services\PaymentService;
use App\Services\PaymentProfileService;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use App\Services\Plugin\HookManager;
use Illuminate\Support\Facades\Crypt;
use Illuminate\Support\Facades\DB;

class PaymentController extends Controller
{
    public function notify($method, $uuid, Request $request)
    {
        HookManager::call('payment.notify.before', [$method, $uuid, $request]);
        try {
            $overrideConfig = null;
            // EPay can be paid via per-admin credentials; verify sign with the correct key snapshot.
            if ((string) $method === 'EPay') {
                $tradeNo = (string) ($request->input('out_trade_no') ?: $request->input('trade_no') ?: '');
                if ($tradeNo !== '') {
                    $order = Order::query()->where('trade_no', $tradeNo)->first();
                    if ($order && $order->epay_pid && $order->epay_url && $order->epay_key_encrypted) {
                        try {
                            $overrideConfig = [
                                'pid' => (string) $order->epay_pid,
                                'key' => Crypt::decryptString((string) $order->epay_key_encrypted),
                                'url' => (string) $order->epay_url,
                                'submit_path' => '/pay/submit.php',
                                'use_post' => true,
                            ];
                        } catch (\Throwable $e) {
                            $overrideConfig = null;
                        }
                    } else {
                        $donation = SponsorDonation::query()->where('trade_no', $tradeNo)->first();
                        if ($donation) {
                            $epay = app(PaymentProfileService::class)->getSponsorEpayProfile();
                            if ($epay) {
                                $overrideConfig = $epay;
                            }
                        }
                    }
                }
            }

            $paymentService = new PaymentService($method, null, $uuid, $overrideConfig);
            $verify = $paymentService->notify($request->input());
            if (!$verify) {
                HookManager::call('payment.notify.failed', [$method, $uuid, $request]);
                return $this->fail([422, 'verify error']);
            }
            HookManager::call('payment.notify.verified', $verify);
            if (!$this->handle($verify['trade_no'], $verify['callback_no'])) {
                return $this->fail([400, 'handle error']);
            }
            return (isset($verify['custom_result']) ? $verify['custom_result'] : 'success');
        } catch (\Exception $e) {
            Log::error($e);
            return $this->fail([500, 'fail']);
        }
    }

    private function handle($tradeNo, $callbackNo)
    {
        $order = Order::query()->where('trade_no', $tradeNo)->first();
        if ($order) {
            $orderService = new OrderService($order);
            if (!$orderService->paid($callbackNo)) {
                return false;
            }
        } else {
            $donation = SponsorDonation::query()->where('trade_no', $tradeNo)->first();
            if (!$donation) {
                return false;
            }

            $updated = DB::transaction(function () use ($donation, $callbackNo) {
                return SponsorDonation::query()
                    ->where('id', $donation->id)
                    ->where('status', SponsorDonation::STATUS_PENDING)
                    ->update([
                        'status' => SponsorDonation::STATUS_COMPLETED,
                        'paid_at' => time(),
                        'callback_no' => $callbackNo,
                    ]);
            });
            if ($updated === 0) {
                return true;
            }
        }

        HookManager::call('payment.notify.success', $order ?? null);
        return true;
    }
}
