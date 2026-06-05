<?php

namespace Plugin\Btcpay;

use App\Services\Plugin\AbstractPlugin;
use App\Contracts\PaymentInterface;
use App\Exceptions\ApiException;

class Plugin extends AbstractPlugin implements PaymentInterface
{
    private const REQUEST_TIMEOUT_SECONDS = 20;
    private const CONNECT_TIMEOUT_SECONDS = 10;

    public function boot(): void
    {
        $this->filter('available_payment_methods', function($methods) {
            if ($this->getConfig('enabled', true)) {
                $methods['BTCPay'] = [
                    'name' => $this->getConfig('display_name', 'BTCPay'),
                    'icon' => $this->getConfig('icon', '₿'),
                    'plugin_code' => $this->getPluginCode(),
                    'type' => 'plugin'
                ];
            }
            return $methods;
        });
    }

    public function form(): array
    {
        return [
            'btcpay_url' => [
                'label' => 'API接口所在网址',
                'type' => 'string',
                'required' => true,
                'description' => '包含最后的斜杠，例如：https://your-btcpay.com/'
            ],
            'btcpay_storeId' => [
                'label' => 'Store ID',
                'type' => 'string',
                'required' => true,
                'description' => 'BTCPay商店标识符'
            ],
            'btcpay_api_key' => [
                'label' => 'API KEY',
                'type' => 'string',
                'required' => true,
                'description' => '个人设置中的API KEY(非商店设置中的)'
            ],
            'btcpay_webhook_key' => [
                'label' => 'WEBHOOK KEY',
                'type' => 'string',
                'required' => true,
                'description' => 'Webhook通知密钥'
            ],
        ];
    }

    public function pay($order): array
    {
        $params = [
            'jsonResponse' => true,
            'amount' => sprintf('%.2f', $order['total_amount'] / 100),
            'currency' => 'CNY',
            'metadata' => [
                'orderId' => $order['trade_no']
            ]
        ];

        $params_string = @json_encode($params);
        $ret_raw = $this->curlRequest('POST', $this->buildInvoiceUrl(), $params_string);
        $ret = @json_decode($ret_raw, true);

        if (empty($ret['checkoutLink'])) {
            throw new ApiException("error!");
        }
        
        return [
            'type' => 1,
            'data' => $ret['checkoutLink'],
        ];
    }

    public function notify($params): array|bool
    {
        $payload = trim(request()->getContent());
        $signraturHeader = (string) request()->header('Btcpay-Sig', '');
        $json_param = json_decode($payload, true);

        $computedSignature = "sha256=" . \hash_hmac('sha256', $payload, $this->getConfig('btcpay_webhook_key'));

        if (!$this->hashEqual($signraturHeader, $computedSignature)) {
            throw new ApiException('HMAC signature does not match', 400);
        }

        if (!is_array($json_param) || empty($json_param['invoiceId']) || !is_scalar($json_param['invoiceId'])) {
            throw new ApiException('Invalid BTCPay payload', 400);
        }

        $invoiceDetail = $this->curlRequest('GET', $this->buildInvoiceDetailUrl((string) $json_param['invoiceId']));
        $invoiceDetail = json_decode($invoiceDetail, true);
        if (!is_array($invoiceDetail) || empty($invoiceDetail['metadata']['orderId'])) {
            throw new ApiException('Invalid BTCPay invoice response', 400);
        }

        $out_trade_no = $invoiceDetail['metadata']["orderId"];
        $pay_trade_no = $json_param['invoiceId'];
        
        return [
            'trade_no' => $out_trade_no,
            'callback_no' => $pay_trade_no
        ];
    }

    private function buildInvoiceUrl(): string
    {
        return $this->buildApiUrl('invoices');
    }

    private function buildInvoiceDetailUrl(string $invoiceId): string
    {
        return $this->buildApiUrl('invoices/' . rawurlencode($invoiceId));
    }

    private function buildApiUrl(string $path): string
    {
        $baseUrl = $this->normalizedBaseUrl();
        $storeId = rawurlencode((string) $this->getConfig('btcpay_storeId'));
        return "{$baseUrl}api/v1/stores/{$storeId}/{$path}";
    }

    private function normalizedBaseUrl(): string
    {
        $url = trim((string) $this->getConfig('btcpay_url'));
        $parts = parse_url($url);
        $scheme = strtolower((string) ($parts['scheme'] ?? ''));

        if (!$parts || !isset($parts['host']) || !in_array($scheme, ['http', 'https'], true)) {
            throw new ApiException('Invalid BTCPay API URL', 400);
        }

        return rtrim($url, '/') . '/';
    }

    private function curlRequest(string $method, string $url, $params = false): string
    {
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $url);
        curl_setopt($ch, CURLOPT_HEADER, false);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_TIMEOUT, self::REQUEST_TIMEOUT_SECONDS);
        curl_setopt($ch, CURLOPT_CONNECTTIMEOUT, self::CONNECT_TIMEOUT_SECONDS);
        if (defined('CURLOPT_PROTOCOLS')) {
            curl_setopt($ch, CURLOPT_PROTOCOLS, CURLPROTO_HTTP | CURLPROTO_HTTPS);
        }
        if (defined('CURLOPT_REDIR_PROTOCOLS')) {
            curl_setopt($ch, CURLOPT_REDIR_PROTOCOLS, CURLPROTO_HTTP | CURLPROTO_HTTPS);
        }
        if ($method === 'POST') {
            curl_setopt($ch, CURLOPT_POST, true);
            curl_setopt($ch, CURLOPT_POSTFIELDS, $params);
        }
        curl_setopt(
            $ch,
            CURLOPT_HTTPHEADER,
            array('Authorization: token ' . $this->getConfig('btcpay_api_key'), 'Content-Type: application/json')
        );
        $result = curl_exec($ch);
        if ($result === false) {
            $error = curl_error($ch);
            curl_close($ch);
            throw new ApiException('BTCPay request failed: ' . $error, 502);
        }
        curl_close($ch);
        return $result;
    }

    private function hashEqual($str1, $str2)
    {
        if (function_exists('hash_equals')) {
            return \hash_equals($str1, $str2);
        }

        if (strlen($str1) != strlen($str2)) {
            return false;
        } else {
            $res = $str1 ^ $str2;
            $ret = 0;

            for ($i = strlen($res) - 1; $i >= 0; $i--) {
                $ret |= ord($res[$i]);
            }
            return !$ret;
        }
    }
}
