<?php

namespace App\Console\Commands;

use App\Services\OrderService;
use Illuminate\Console\Command;
use App\Models\Order;

class CheckOrder extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'check:order';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = '订单检查任务';

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
        ini_set('memory_limit', (string) env('APP_MEMORY_LIMIT', '512M'));
        $orders = Order::whereIn('status', [Order::STATUS_PENDING, Order::STATUS_PROCESSING])
            ->orderBy('created_at', 'ASC')
            ->get();
        foreach ($orders as $order) {
            OrderService::handleTradeNo((string) $order->trade_no);
        }
    }
}
