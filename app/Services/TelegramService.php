<?php

namespace App\Services;

use App\Exceptions\ApiException;
use App\Jobs\SendTelegramJob;
use App\Models\User;
use App\Services\Plugin\HookManager;
use Illuminate\Http\Client\PendingRequest;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Throwable;

class TelegramService
{
    protected PendingRequest $http;
    protected string $apiUrl;
    protected string $botToken;

    public function __construct(?string $token = null)
    {
        $this->botToken = (string) admin_setting('telegram_bot_token', $token);
        $this->apiUrl = "https://api.telegram.org/bot{$this->botToken}/";

        $this->http = Http::timeout(30)
            ->retry(3, 1000)
            ->withHeaders([
                'Accept' => 'application/json',
            ]);
    }

    public function sendMessage(int $chatId, string $text, string $parseMode = '', array $options = []): object
    {
        return $this->request('sendMessage', array_merge($this->normalizeMessageOptions($options), [
            'chat_id' => $chatId,
            'text' => $parseMode === 'markdown' ? str_replace('_', '\_', $text) : $text,
            'parse_mode' => $parseMode ?: null,
        ]));
    }

    public function sendContextMessage(
        int $chatId,
        string $text,
        array $context,
        string $parseMode = 'markdown',
        array $options = []
    ): object {
        $response = $this->sendMessage($chatId, $text, $parseMode, $options);
        $messageId = (int) ($response->result->message_id ?? 0);

        if ($messageId > 0) {
            app(TelegramMessageContextService::class)->put($chatId, $messageId, $context);
        }

        return $response;
    }

    public function editMessageText(
        int $chatId,
        int $messageId,
        string $text,
        string $parseMode = '',
        array $options = []
    ): object {
        return $this->request('editMessageText', array_merge($this->normalizeMessageOptions($options), [
            'chat_id' => $chatId,
            'message_id' => $messageId,
            'text' => $parseMode === 'markdown' ? str_replace('_', '\_', $text) : $text,
            'parse_mode' => $parseMode ?: null,
        ]));
    }

    public function answerCallbackQuery(string $callbackQueryId, ?string $text = null, bool $showAlert = false): object
    {
        return $this->request('answerCallbackQuery', [
            'callback_query_id' => $callbackQueryId,
            'text' => $text ?: null,
            'show_alert' => $showAlert,
        ]);
    }

    public function approveChatJoinRequest(int $chatId, int $userId): void
    {
        $this->request('approveChatJoinRequest', [
            'chat_id' => $chatId,
            'user_id' => $userId,
        ]);
    }

    public function declineChatJoinRequest(int $chatId, int $userId): void
    {
        $this->request('declineChatJoinRequest', [
            'chat_id' => $chatId,
            'user_id' => $userId,
        ]);
    }

    public function getMe(): object
    {
        return $this->request('getMe');
    }

    public function setWebhook(string $url): object
    {
        $result = $this->request('setWebhook', [
            'url' => $url,
            'secret_token' => $this->webhookSecretToken(),
        ]);
        return $result;
    }

    /**
     * 注册 Bot 命令列表
     */
    public function registerBotCommands(): void
    {
        try {
            $commands = HookManager::filter('telegram.bot.commands', []);

            if (empty($commands)) {
                Log::warning('没有找到任何 Telegram Bot 命令');
                return;
            }

            $this->request('setMyCommands', [
                'commands' => json_encode($commands),
                'scope' => json_encode(['type' => 'default'])
            ]);

            Log::info('Telegram Bot 命令注册成功', [
                'commands_count' => count($commands),
                'commands' => $commands
            ]);

        } catch (\Exception $e) {
            Log::error('Telegram Bot 命令注册失败', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString()
            ]);
        }
    }

    /**
     * 获取当前注册的命令列表
     */
    public function getMyCommands(): object
    {
        return $this->request('getMyCommands');
    }

    /**
     * 删除所有命令
     */
    public function deleteMyCommands(): object
    {
        return $this->request('deleteMyCommands');
    }

    public function queueMessage(
        int $chatId,
        string $text,
        string $parseMode = 'markdown',
        array $options = [],
        ?array $context = null
    ): void {
        if ((bool) config('ops.telegram_sync_send', false)) {
            if (is_array($context) && !empty($context)) {
                $this->sendContextMessage($chatId, $text, $context, $parseMode, $options);
                return;
            }

            $this->sendMessage($chatId, $text, $parseMode, $options);
            return;
        }

        SendTelegramJob::dispatch($chatId, $text, $parseMode, $options, $context);
    }

    public function sendMessageWithAdmin(
        string $message,
        bool $isStaff = false,
        string $parseMode = 'markdown',
        array $options = [],
        ?array $context = null
    ): void
    {
        $query = User::where('telegram_id', '!=', null);
        $query->where(
            fn($q) => $q->where('is_admin', 1)
                ->orWhere('is_super_admin', 1)
                ->when($isStaff, fn($q) => $q->orWhere('is_staff', 1))
        );
        $this->queueMessageForUsers($query->get(), $message, $parseMode, $options, $context);
    }

    public function sendOpsAlert(string $message, array $context = [], bool $isStaff = false): void
    {
        if (!(bool) admin_setting('telegram_bot_enable', 0)) {
            return;
        }

        if (!(bool) admin_setting('telegram_notify_ops_alert', 1)) {
            return;
        }

        $text = $this->formatOpsAlertMessage($message, $context);

        try {
            $this->sendMessageWithAdmin($text, $isStaff, 'markdown', [], $context ?: null);
        } catch (Throwable $exception) {
            Log::warning('Telegram 运维告警发送失败', [
                'error' => $exception->getMessage(),
                'context' => $context,
            ]);
        }
    }

    public function queueMessageForUsers(
        iterable $users,
        string $message,
        string $parseMode = 'markdown',
        array $options = [],
        ?array $context = null
    ): void {
        Collection::make($users)
            ->filter(fn ($user) => !empty($user?->telegram_id))
            ->unique(fn ($user) => (int) $user->telegram_id)
            ->each(function ($user) use ($message, $parseMode, $options, $context): void {
                $this->queueMessage((int) $user->telegram_id, $message, $parseMode, $options, $context);
            });
    }

    protected function request(string $method, array $params = []): object
    {
        try {
            $params = $this->normalizeRequestParams($params);
            $response = empty($params)
                ? $this->http->get($this->apiUrl . $method)
                : $this->http->asForm()->post($this->apiUrl . $method, $params);

            if (!$response->successful()) {
                throw new ApiException("HTTP 请求失败: {$response->status()}");
            }

            $data = $response->object();

            if (!isset($data->ok)) {
                throw new ApiException('无效的 Telegram API 响应');
            }

            if (!$data->ok) {
                $description = $data->description ?? '未知错误';
                throw new ApiException("Telegram API 错误: {$description}");
            }

            return $data;

        } catch (\Exception $e) {
            Log::error('Telegram API 请求失败', [
                'method' => $method,
                'params' => $this->sanitizeRequestParamsForLogs($params),
                'error' => $e->getMessage(),
            ]);

            throw new ApiException("Telegram 服务错误: {$e->getMessage()}");
        }
    }

    protected function normalizeRequestParams(array $params): array
    {
        return collect($params)
            ->reject(fn ($value) => $value === null || $value === '')
            ->map(function ($value, string $key) {
                if ($key === 'reply_markup' && is_array($value)) {
                    return json_encode($value, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
                }

                return is_bool($value) ? ($value ? 'true' : 'false') : $value;
            })
            ->all();
    }

    protected function normalizeMessageOptions(array $options): array
    {
        $normalized = [];

        foreach ([
            'reply_markup',
            'reply_to_message_id',
            'disable_notification',
            'disable_web_page_preview',
        ] as $field) {
            if (array_key_exists($field, $options)) {
                $normalized[$field] = $options[$field];
            }
        }

        return $normalized;
    }

    public function webhookSecretToken(): string
    {
        return hash_hmac(
            'sha256',
            $this->botToken,
            (string) config('app.key')
        );
    }

    public function webhookSecretMatches(?string $providedSecret): bool
    {
        $providedSecret = trim((string) $providedSecret);

        return $providedSecret !== '' && hash_equals($this->webhookSecretToken(), $providedSecret);
    }

    private function formatOpsAlertMessage(string $message, array $context): string
    {
        $header = str_contains($message, '[Ops Alert]') ? '' : "*[Ops Alert]*\n";
        $contextLines = collect($context)
            ->map(function ($value, string $key): string {
                if (is_scalar($value) || $value === null) {
                    return "{$key}: " . (string) $value;
                }

                return "{$key}: " . json_encode($value, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
            })
            ->implode("\n");

        if ($contextLines === '') {
            return $header . trim($message);
        }

        return $header . trim($message) . "\n\n" . $contextLines;
    }

    private function sanitizeRequestParamsForLogs(array $params): array
    {
        foreach (['secret_token', 'access_token', 'token'] as $sensitiveKey) {
            if (array_key_exists($sensitiveKey, $params) && $params[$sensitiveKey] !== null && $params[$sensitiveKey] !== '') {
                $params[$sensitiveKey] = '***';
            }
        }

        return $params;
    }
}
