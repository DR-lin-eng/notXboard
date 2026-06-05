<?php

namespace Tests\Unit;

use App\Services\PaymentService;
use Tests\TestCase;

class PaymentServiceReturnUrlTest extends TestCase
{
    public function test_default_return_url_points_to_app_spa(): void
    {
        config(['app.url' => 'https://portal.example.com']);

        $service = new class extends PaymentService {
            public function __construct()
            {
                $this->method = 'stub';
                $this->config = [
                    'uuid' => 'stub-uuid',
                    'enable' => 1,
                    'notify_domain' => '',
                ];
                $this->payment = new class {
                    public function pay(array $payload): array
                    {
                        return $payload;
                    }
                };
            }
        };

        $result = $service->pay([
            'trade_no' => '202604180001',
            'total_amount' => 1000,
            'user_id' => 1,
            'stripe_token' => null,
        ]);

        $this->assertSame(
            'https://portal.example.com/app#/order/202604180001',
            $result['return_url']
        );
    }
}
