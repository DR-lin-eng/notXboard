<?php

namespace App\Console\Commands;

use App\Models\CommissionLog;
use Illuminate\Console\Command;
use App\Models\Order;
use App\Models\User;
use Illuminate\Support\Facades\DB;

class CheckCommission extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'check:commission';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = '返佣服务';

    /**
     * Create a new command instance.
     *
     * @return void
     */
    public function __construct()
    {
        parent::__construct();
    }

    /**
     * Execute the console command.
     *
     * @return mixed
     */
    public function handle()
    {
        $this->autoCheck();
        $this->autoPayCommission();
    }

    public function autoCheck()
    {
        if ((int)admin_setting('commission_auto_check_enable', 1)) {
            Order::where('commission_status', 0)
                ->where('invite_user_id', '!=', NULL)
                ->where('status', 3)
                ->where('updated_at', '<=', strtotime('-3 day', time()))
                ->update([
                    'commission_status' => 1
                ]);
        }
    }

    public function autoPayCommission()
    {
        $orderIds = Order::where('commission_status', 1)
            ->where('invite_user_id', '!=', NULL)
            ->pluck('id');

        foreach ($orderIds as $orderId) {
            DB::transaction(function () use ($orderId) {
                $order = Order::query()->lockForUpdate()->find($orderId);
                if (!$order || (int) $order->commission_status !== 1 || !$order->invite_user_id) {
                    return;
                }

                if (!$this->payHandle($order->invite_user_id, $order)) {
                    throw new \RuntimeException('Commission payout failed.');
                }

                $order->commission_status = 2;
                if (!$order->save()) {
                    throw new \RuntimeException('Failed to update commission status.');
                }
            });
        }
    }

    public function payHandle($inviteUserId, Order $order)
    {
        $level = 3;
        if ((int)admin_setting('commission_distribution_enable', 0)) {
            $commissionShareLevels = [
                0 => (int)admin_setting('commission_distribution_l1'),
                1 => (int)admin_setting('commission_distribution_l2'),
                2 => (int)admin_setting('commission_distribution_l3')
            ];
        } else {
            $commissionShareLevels = [
                0 => 100
            ];
        }
        for ($l = 0; $l < $level; $l++) {
            $inviter = User::query()->lockForUpdate()->find($inviteUserId);
            if (!$inviter) continue;
            if (!isset($commissionShareLevels[$l])) continue;
            $commissionBalance = (int) round($order->commission_balance * ($commissionShareLevels[$l] / 100));
            if (!$commissionBalance) continue;
            $existingLog = CommissionLog::query()
                ->where('invite_user_id', $inviteUserId)
                ->where('user_id', $order->user_id)
                ->where('trade_no', $order->trade_no)
                ->lockForUpdate()
                ->first();
            if ($existingLog) {
                $inviteUserId = $inviter->invite_user_id;
                continue;
            }
            if ((int)admin_setting('withdraw_close_enable', 0)) {
                $inviter->balance = $inviter->balance + $commissionBalance;
            } else {
                $inviter->commission_balance = $inviter->commission_balance + $commissionBalance;
            }
            if (!$inviter->save()) {
                return false;
            }

            CommissionLog::query()->create([
                'invite_user_id' => $inviteUserId,
                'user_id' => $order->user_id,
                'trade_no' => $order->trade_no,
                'order_amount' => $order->total_amount,
                'get_amount' => $commissionBalance,
            ]);

            $inviteUserId = $inviter->invite_user_id;
            // update order actual commission balance
            $order->actual_commission_balance = (int) ($order->actual_commission_balance ?? 0) + $commissionBalance;
        }
        return true;
    }

}
