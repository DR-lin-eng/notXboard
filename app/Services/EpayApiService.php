<?php

namespace App\Services;

use App\Support\UrlSecurity;
use Illuminate\Support\Facades\Http;

class EpayApiService
{
    public function refund(string $baseUrl, string $pid, string $key, string $gatewayTradeNo, string $money, ?string $outTradeNo = null): array
    {
        try {
            $url = UrlSecurity::normalizeHttpUrl($baseUrl, true) . '/api.php';
        } catch (\InvalidArgumentException $e) {
            return ['ok' => false, 'message' => $e->getMessage(), 'raw' => null];
        }

        $payload = [
            'pid' => $pid,
            'key' => $key,
            'trade_no' => $gatewayTradeNo,
            'money' => $money,
        ];
        if ($outTradeNo !== null && $outTradeNo !== '') {
            $payload['out_trade_no'] = $outTradeNo;
        }

        $res = Http::asForm()->timeout(20)->post($url, $payload);
        $json = $res->json();
        if (!is_array($json)) {
            return ['ok' => false, 'message' => 'Invalid gateway response', 'raw' => $res->body()];
        }
        if ((int) ($json['code'] ?? 0) !== 1) {
            return ['ok' => false, 'message' => (string) ($json['msg'] ?? 'Refund failed'), 'raw' => $json];
        }
        return ['ok' => true, 'message' => (string) ($json['msg'] ?? 'ok'), 'raw' => $json];
    }
}
