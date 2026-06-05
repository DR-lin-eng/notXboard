<?php

namespace Plugin\Epay;

use App\Services\Plugin\AbstractPlugin;
use App\Contracts\PaymentInterface;
use App\Support\UrlSecurity;

class Plugin extends AbstractPlugin implements PaymentInterface
{
    public function boot(): void
    {
        $this->filter('available_payment_methods', function ($methods) {
            if ($this->getConfig('enabled', true)) {
                $methods['EPay'] = [
                    'name' => $this->getConfig('display_name', '易支付'),
                    'icon' => $this->getConfig('icon', '💳'),
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
            'url' => [
                'label' => '支付网关地址',
                'type' => 'string',
                'required' => true,
                'description' => '请填写完整的支付网关地址，包括协议（http或https）'
            ],
            'submit_path' => [
                'label' => '提交接口路径',
                'type' => 'string',
                'required' => false,
                'description' => '默认 /submit.php；credit.linux.do 为 /pay/submit.php'
            ],
            'pid' => [
                'label' => '商户ID',
                'type' => 'string',
                'description' => '请填写商户ID',
                'required' => true
            ],
            'key' => [
                'label' => '通信密钥',
                'type' => 'string',
                'required' => true,
                'description' => '请填写通信密钥'
            ],
            'use_post' => [
                'label' => '使用POST跳转',
                'type' => 'boolean',
                'required' => false,
                'description' => '部分易支付需要 POST 提交；credit.linux.do 必须使用 POST（建议开启），开启后将返回自动提交表单'
            ],
            'sitename' => [
                'label' => '站点名称（可选）',
                'type' => 'string',
                'required' => false,
                'description' => '部分易支付会展示该字段'
            ],
            'device' => [
                'label' => '终端标识（可选）',
                'type' => 'string',
                'required' => false,
                'description' => '部分服务支持 device 字段'
            ],
        ];
    }

    public function pay($order): array
    {
        $money = (float) ($order['total_amount'] ?? 0) / 100;
        $money = number_format($money, 2, '.', '');

        $params = [
            'pid' => (string) $this->getConfig('pid'),
            'type' => 'epay',
            'out_trade_no' => (string) $order['trade_no'],
            'name' => (string) ($order['trade_no'] ?? 'order'),
            'money' => (string) $money,
            // Per credit.linux.do spec, notify_url/return_url in request participate in sign but do not override app settings.
            'notify_url' => (string) ($order['notify_url'] ?? ''),
            'return_url' => (string) ($order['return_url'] ?? ''),
        ];

        if ($sitename = $this->getConfig('sitename')) {
            $params['sitename'] = (string) $sitename;
        }
        if ($device = $this->getConfig('device')) {
            $params['device'] = (string) $device;
        }

        // Sign: all non-empty fields excluding sign/sign_type, ASCII sort, k=v&k=v, append secret, md5 lower.
        $paramsForSign = array_filter($params, function ($v) {
            return $v !== null && $v !== '';
        });
        ksort($paramsForSign);
        $pairs = [];
        foreach ($paramsForSign as $k => $v) {
            $pairs[] = $k . '=' . $v;
        }
        $payload = implode('&', $pairs);
        $params['sign'] = md5($payload . (string) $this->getConfig('key'));
        $params['sign_type'] = 'MD5';

        $baseUrl = UrlSecurity::normalizeHttpUrl((string) $this->getConfig('url'));
        $submitPath = UrlSecurity::normalizeRelativePath((string) $this->getConfig('submit_path'), '/submit.php');
        $submitUrl = $baseUrl . $submitPath;

        if ((bool) $this->getConfig('use_post', false)) {
            $inputs = '';
            foreach ($params as $k => $v) {
                $kEsc = htmlspecialchars((string) $k, ENT_QUOTES);
                $vEsc = htmlspecialchars((string) $v, ENT_QUOTES);
                $inputs .= "<input type=\"hidden\" name=\"{$kEsc}\" value=\"{$vEsc}\">";
            }
            $submitUrlEsc = htmlspecialchars($submitUrl, ENT_QUOTES);
            $html = "<form id=\"epay_submit\" action=\"{$submitUrlEsc}\" method=\"post\">{$inputs}</form>"
                . "<script>document.getElementById('epay_submit').submit();</script>";

            return [
                'type' => 1,
                'data' => $html,
            ];
        }

        return [
            'type' => 1,
            'data' => $submitUrl . '?' . http_build_query($params)
        ];
    }

    public function notify($params): array|bool
    {
        if (isset($params['trade_status'])) {
            $status = strtoupper((string) $params['trade_status']);
            $ok = in_array($status, ['TRADE_SUCCESS', 'TRADE_FINISHED', 'SUCCESS'], true);
            if (!$ok) {
                return false;
            }
        }

        $sign = (string) ($params['sign'] ?? '');
        unset($params['sign'], $params['sign_type']);
        $paramsForSign = array_filter($params, function ($v) {
            return $v !== null && $v !== '';
        });
        ksort($paramsForSign);
        $pairs = [];
        foreach ($paramsForSign as $k => $v) {
            $pairs[] = $k . '=' . $v;
        }
        $payload = implode('&', $pairs);
        $expected = md5($payload . (string) $this->getConfig('key'));

        if (!hash_equals($expected, $sign)) {
            return false;
        }

        return [
            'trade_no' => $params['out_trade_no'],
            'callback_no' => $params['trade_no']
        ];
    }
}
