<?php

namespace Plugin\Telegram;

use App\Models\Notice;
use App\Models\Order;
use App\Models\OrderRefundRequest;
use App\Models\Ticket;
use App\Models\TicketMessage;
use App\Models\User;
use App\Models\UserPlanSubscription;
use App\Services\Plugin\AbstractPlugin;
use App\Services\Plugin\HookManager;
use App\Services\TelegramMessageContextService;
use App\Services\TelegramService;
use App\Services\TicketService;
use App\Utils\Helper;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Log;

class Plugin extends AbstractPlugin
{
    protected array $commands = [];
    protected TelegramService $telegramService;
    protected TicketService $ticketService;
    protected TelegramMessageContextService $contextService;

    protected array $commandConfigs = [
        '/start' => ['description' => '开始使用', 'handler' => 'handleStartCommand'],
        '/bind' => ['description' => '绑定账号', 'handler' => 'handleBindCommand'],
        '/traffic' => ['description' => '查看流量', 'handler' => 'handleTrafficCommand'],
        '/getlatesturl' => ['description' => '获取订阅链接', 'handler' => 'handleGetLatestUrlCommand'],
        '/tickets' => ['description' => '查看工单', 'handler' => 'handleTicketsCommand'],
        '/ticket' => ['description' => '查看工单详情', 'handler' => 'handleTicketCommand'],
        '/close' => ['description' => '关闭工单', 'handler' => 'handleCloseCommand'],
        '/unbind' => ['description' => '解绑账号', 'handler' => 'handleUnbindCommand'],
    ];

    public function boot(): void
    {
        $this->telegramService = new TelegramService();
        $this->ticketService = app(TicketService::class);
        $this->contextService = app(TelegramMessageContextService::class);

        $this->registerDefaultCommands();

        $this->filter('telegram.message.handle', [$this, 'handleMessage'], 10);
        $this->listen('telegram.message.unhandled', [$this, 'handleUnknownCommand'], 10);
        $this->listen('telegram.message.error', [$this, 'handleError'], 10);
        $this->filter('telegram.bot.commands', [$this, 'addBotCommands'], 10);

        $this->listen('ticket.create.after', [$this, 'handleTicketCreated'], 10);
        $this->listen('ticket.reply.user.after', [$this, 'handleTicketUserReplied'], 10);
        $this->listen('ticket.reply.admin.after', [$this, 'handleTicketAdminReplied'], 10);
        $this->listen('ticket.close.after', [$this, 'handleTicketClosed'], 10);
        $this->listen('payment.notify.success', [$this, 'handlePaymentNotify'], 10);
        $this->listen('notice.published', [$this, 'handleNoticePublished'], 10);
        $this->listen('tcping.alert.triggered', [$this, 'handleTcpingAlertTriggered'], 10);
        $this->listen('tcping.alert.recovered', [$this, 'handleTcpingAlertRecovered'], 10);
        $this->listen('refund.request.created', [$this, 'handleRefundRequestCreated'], 10);
        $this->listen('refund.vote.started', [$this, 'handleRefundVoteStarted'], 10);
        $this->listen('refund.vote.cast', [$this, 'handleRefundVoteCast'], 10);
        $this->listen('refund.status.changed', [$this, 'handleRefundStatusChanged'], 10);
    }

    protected function registerDefaultCommands(): void
    {
        foreach ($this->commandConfigs as $command => $config) {
            $this->registerTelegramCommand($command, [$this, $config['handler']]);
        }

        $this->registerReplyHandler('/(工单\\s*#|工单ID[:：]?\\s*)(\\d+)/u', [$this, 'handleLegacyTicketReply']);
    }

    public function registerTelegramCommand(string $command, callable $handler): void
    {
        $this->commands['commands'][$command] = $handler;
    }

    public function registerReplyHandler(string $regex, callable $handler): void
    {
        $this->commands['replies'][$regex] = $handler;
    }

    public function handleMessage(bool $handled, array $data): bool
    {
        [$msg] = $data;
        if ($handled || !(bool) admin_setting('telegram_bot_enable', 0)) {
            return $handled;
        }

        try {
            return match ($msg->message_type) {
                'message' => $this->handleCommandMessage($msg),
                'reply_message' => $this->handleReplyMessage($msg),
                'callback_query' => $this->handleCallbackMessage($msg),
                default => false,
            };
        } catch (\Throwable $e) {
            Log::error('Telegram 命令处理意外错误', [
                'command' => $msg->command ?? 'unknown',
                'chat_id' => $msg->chat_id ?? 'unknown',
                'message_type' => $msg->message_type ?? 'unknown',
                'error' => $e->getMessage(),
                'file' => $e->getFile(),
                'line' => $e->getLine(),
            ]);

            if (!empty($msg->callback_query_id)) {
                rescue(fn () => $this->telegramService->answerCallbackQuery($msg->callback_query_id, $this->limitText($e->getMessage(), 60), true), report: false);
            }

            if (isset($msg->chat_id)) {
                $this->telegramService->sendMessage($msg->chat_id, $e->getMessage());
            }

            return true;
        }
    }

    protected function handleCommandMessage(object $msg): bool
    {
        if (!isset($this->commands['commands'][$msg->command])) {
            return false;
        }

        call_user_func($this->commands['commands'][$msg->command], $msg);
        return true;
    }

    protected function handleReplyMessage(object $msg): bool
    {
        if (!$this->checkPrivateChat($msg)) {
            return true;
        }

        $replyMessageId = (int) ($msg->reply_message_id ?? 0);
        if ($replyMessageId > 0) {
            $context = $this->contextService->get((int) $msg->chat_id, $replyMessageId);
            if (($context['type'] ?? null) === 'ticket') {
                $this->handleTicketReplyByContext($msg, $context);
                return true;
            }
        }

        if (!isset($this->commands['replies'])) {
            return false;
        }

        foreach ($this->commands['replies'] as $regex => $handler) {
            if (preg_match($regex, (string) ($msg->reply_text ?? ''), $matches)) {
                call_user_func($handler, $msg, $matches);
                return true;
            }
        }

        return false;
    }

    protected function handleCallbackMessage(object $msg): bool
    {
        if (!$this->checkPrivateChat($msg)) {
            if (!empty($msg->callback_query_id)) {
                $this->telegramService->answerCallbackQuery($msg->callback_query_id, '请在私聊中使用');
            }
            return true;
        }

        $user = $this->getBoundUser($msg);
        if (!$user) {
            if (!empty($msg->callback_query_id)) {
                $this->telegramService->answerCallbackQuery($msg->callback_query_id, '请先绑定账号');
            }
            return true;
        }

        $parts = explode(':', (string) ($msg->callback_data ?? ''));
        if (($parts[0] ?? '') !== 'tk') {
            return false;
        }

        $action = $parts[1] ?? '';
        if ($action === 'list') {
            $scope = $parts[2] ?? 'mine';
            $this->sendTicketListMessage($msg, $user, $scope);
            $this->telegramService->answerCallbackQuery($msg->callback_query_id, '已刷新工单列表');
            return true;
        }

        if ($action === 'view') {
            $ticketId = (int) ($parts[2] ?? 0);
            $ticket = Ticket::query()->with(['user', 'node', 'assignedAdmin'])->find($ticketId);
            if (!$ticket) {
                $this->telegramService->answerCallbackQuery($msg->callback_query_id, '工单不存在', true);
                return true;
            }

            $this->sendTicketDetailMessage($msg, $ticket, $user);
            $this->telegramService->answerCallbackQuery($msg->callback_query_id, '已打开工单详情');
            return true;
        }

        if ($action === 'close') {
            $ticketId = (int) ($parts[2] ?? 0);
            $ticket = Ticket::query()->with(['user', 'node', 'assignedAdmin'])->find($ticketId);
            if (!$ticket) {
                $this->telegramService->answerCallbackQuery($msg->callback_query_id, '工单不存在', true);
                return true;
            }

            $this->ticketService->closeByActor($ticket, $user);
            $this->telegramService->answerCallbackQuery($msg->callback_query_id, '工单已关闭');
            $this->sendTicketDetailMessage($msg, $ticket->fresh(['user', 'node', 'assignedAdmin']), $user, '工单已关闭。');
            return true;
        }

        return false;
    }

    public function handleUnknownCommand(array $data): void
    {
        [$msg] = $data;
        if (!$msg->is_private || $msg->message_type !== 'message') {
            return;
        }

        $helpText = $this->getConfig('help_text', "未知命令，可用命令：\n/start\n/bind 订阅链接\n/traffic\n/getlatesturl\n/tickets\n/ticket 工单ID\n/close 工单ID\n/unbind");
        $this->telegramService->sendMessage($msg->chat_id, str_replace('\\n', "\n", $helpText));
    }

    public function handleError(array $data): void
    {
        [$msg, $e] = $data;
        Log::error('Telegram 消息处理错误', [
            'chat_id' => $msg->chat_id ?? 'unknown',
            'command' => $msg->command ?? 'unknown',
            'message_type' => $msg->message_type ?? 'unknown',
            'error' => $e->getMessage(),
            'file' => $e->getFile(),
            'line' => $e->getLine(),
        ]);

        if (!empty($msg->callback_query_id)) {
            rescue(fn () => $this->telegramService->answerCallbackQuery($msg->callback_query_id, '处理失败，请稍后重试', true), report: false);
        }
    }

    public function handleStartCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $welcomeTitle = str_replace('\\n', "\n", (string) $this->getConfig('start_welcome_title', '欢迎使用 XBoard Telegram Bot'));
        $botDescription = str_replace('\\n', "\n", (string) $this->getConfig('start_bot_description', '我是您的专属助手，可以帮助您绑定账号、查看流量、获取订阅链接和处理工单。'));
        $footer = str_replace('\\n', "\n", (string) $this->getConfig('start_footer', '提示：请在私聊中使用所有命令。'));

        $text = $welcomeTitle . "\n\n" . $botDescription . "\n\n";
        $user = User::query()->where('telegram_id', $msg->chat_id)->first();
        if ($user) {
            $commandLines = [
                '/traffic',
                '/getlatesturl',
            ];

            if (!empty($this->buildTicketMenuButtons($user))) {
                $commandLines[] = '/tickets';
                $commandLines[] = '/ticket 工单ID';
                $commandLines[] = '/close 工单ID';
            }

            $commandLines[] = '/unbind';
            $text .= "已绑定账号：{$user->email}\n";
            $text .= "可用命令：\n" . implode("\n", $commandLines);
        } else {
            $text .= str_replace('\\n', "\n", (string) $this->getConfig('start_bind_guide', "请先绑定您的 XBoard 账号：\n1. 登录站点\n2. 复制订阅链接\n3. 发送 /bind + 订阅链接"));
            $text .= "\n\n/bind [订阅链接] - 绑定账号";
        }
        $text .= "\n\n" . $footer;

        $options = [];
        if ($user) {
            $buttons = $this->buildTicketMenuButtons($user);
            if (!empty($buttons)) {
                $options['reply_markup'] = ['inline_keyboard' => $buttons];
            }
        }

        $this->telegramService->sendMessage($msg->chat_id, $text, '', $options);
    }

    public function handleBindCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $subscribeUrl = $msg->args[0] ?? null;
        if (!$subscribeUrl) {
            $this->sendMessage($msg, '参数有误，请携带订阅地址发送。');
            return;
        }

        $token = $this->extractTokenFromUrl($subscribeUrl);
        if (!$token) {
            $this->sendMessage($msg, '订阅地址无效。');
            return;
        }

        $user = User::query()->where('token', $token)->first();
        if (!$user) {
            $this->sendMessage($msg, '用户不存在。');
            return;
        }

        if ($user->telegram_id && (int) $user->telegram_id !== (int) $msg->chat_id) {
            $this->sendMessage($msg, '该账号已经绑定了其他 Telegram 账号。');
            return;
        }

        $user->telegram_id = $msg->chat_id;
        if (!$user->save()) {
            $this->sendMessage($msg, '设置失败。');
            return;
        }

        HookManager::call('user.telegram.bind.after', [$user]);
        $this->sendMessage($msg, "绑定成功。\n账号：{$user->email}\n可用命令：/traffic /getlatesturl /start");
    }

    public function handleTrafficCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $user = $this->getBoundUser($msg);
        if (!$user) {
            return;
        }

        $transferUsed = (float) ($user->u + $user->d);
        $transferTotal = (float) $user->transfer_enable;
        $transferRemaining = max(0, $transferTotal - $transferUsed);
        $usagePercentage = $transferTotal > 0 ? ($transferUsed / $transferTotal) * 100 : 0;

        $text = sprintf(
            "流量使用情况\n\n已用流量：%s GB\n总流量：%s GB\n剩余流量：%s GB\n使用率：%.2f%%",
            $this->transferToGBString($transferUsed),
            $this->transferToGBString($transferTotal),
            $this->transferToGBString($transferRemaining),
            $usagePercentage
        );

        $this->sendMessage($msg, $text);
    }

    public function handleGetLatestUrlCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $user = $this->getBoundUser($msg);
        if (!$user) {
            return;
        }

        $subscribeUrl = Helper::getSubscribeUrl($user->token);
        $this->sendMessage($msg, "您的订阅链接：\n\n{$subscribeUrl}");
    }

    public function handleTicketsCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $user = $this->getBoundUser($msg);
        if (!$user) {
            return;
        }

        $buttons = $this->buildTicketMenuButtons($user);
        if (empty($buttons)) {
            $this->sendMessage($msg, '当前账号未启用 Telegram 工单处理。');
            return;
        }

        $this->telegramService->sendMessage($msg->chat_id, '请选择要查看的工单视图。', '', [
            'reply_markup' => ['inline_keyboard' => $buttons],
        ]);
    }

    public function handleTicketCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $user = $this->getBoundUser($msg);
        if (!$user) {
            return;
        }

        $ticketId = (int) ($msg->args[0] ?? 0);
        if ($ticketId <= 0) {
            $this->sendMessage($msg, '用法：/ticket 工单ID');
            return;
        }

        $ticket = Ticket::query()->with(['user', 'node', 'assignedAdmin'])->find($ticketId);
        if (!$ticket) {
            $this->sendMessage($msg, '工单不存在。');
            return;
        }

        $this->sendTicketDetailMessage($msg, $ticket, $user);
    }

    public function handleCloseCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $user = $this->getBoundUser($msg);
        if (!$user) {
            return;
        }

        $ticketId = (int) ($msg->args[0] ?? 0);
        if ($ticketId <= 0) {
            $this->sendMessage($msg, '用法：/close 工单ID');
            return;
        }

        $ticket = Ticket::query()->with(['user', 'node', 'assignedAdmin'])->find($ticketId);
        if (!$ticket) {
            $this->sendMessage($msg, '工单不存在。');
            return;
        }

        $this->ticketService->closeByActor($ticket, $user);
        $this->sendMessage($msg, "工单 #{$ticketId} 已关闭。");
    }

    public function handleUnbindCommand(object $msg): void
    {
        if (!$this->checkPrivateChat($msg)) {
            return;
        }

        $user = $this->getBoundUser($msg);
        if (!$user) {
            return;
        }

        $user->telegram_id = null;
        if (!$user->save()) {
            $this->sendMessage($msg, '解绑失败。');
            return;
        }

        $this->sendMessage($msg, '解绑成功。');
    }

    public function handleLegacyTicketReply(object $msg, array $matches): void
    {
        $ticketId = isset($matches[2]) ? (int) $matches[2] : 0;
        if ($ticketId <= 0) {
            $this->sendMessage($msg, '未能识别工单 ID，请直接回复包含工单详情的机器人消息。');
            return;
        }

        $this->handleTicketReplyByContext($msg, [
            'type' => 'ticket',
            'ticket_id' => $ticketId,
        ]);
    }

    public function handleTicketCreated(Ticket $ticket): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_ticket_created', true)) {
            return;
        }

        $ticket = $ticket->fresh(['user', 'node', 'assignedAdmin', 'messages.user']);
        if (!$ticket) {
            return;
        }

        $latestMessage = $ticket->messages()->latest('id')->first();
        $adminRecipients = $this->getTicketStaffRecipients($ticket);
        $adminMessage = $this->buildTicketNoticeMessage($ticket, '新工单待处理', $latestMessage, true);
        $this->telegramService->queueMessageForUsers(
            $adminRecipients,
            $adminMessage,
            '',
            ['reply_markup' => ['inline_keyboard' => $this->buildTicketActionButtons($ticket)]],
            ['type' => 'ticket', 'ticket_id' => (int) $ticket->id]
        );

        $owner = $ticket->user;
        if ($owner && !empty($owner->telegram_id)) {
            $message = "工单 #{$ticket->id} 已创建。\n主题：{$ticket->subject}\n";
            $message .= $this->shouldAllowUserTicketActions($owner, $ticket)
                ? "如需补充信息，可直接回复此消息。"
                : "您将通过 Telegram 收到后续处理通知。";

            if ($this->shouldAllowUserTicketActions($owner, $ticket)) {
                $this->telegramService->queueMessage((int) $owner->telegram_id, $message, '', [
                    'reply_markup' => ['inline_keyboard' => $this->buildTicketActionButtons($ticket)],
                ], ['type' => 'ticket', 'ticket_id' => (int) $ticket->id]);
            } else {
                $this->telegramService->queueMessage((int) $owner->telegram_id, $message, '');
            }
        }
    }

    public function handleTicketUserReplied(Ticket $ticket): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_ticket_replied', true)) {
            return;
        }

        $ticket = $ticket->fresh(['user', 'node', 'assignedAdmin']);
        if (!$ticket) {
            return;
        }

        $latestMessage = $ticket->messages()->latest('id')->first();
        $message = $this->buildTicketNoticeMessage($ticket, '用户已回复工单', $latestMessage, true);
        $this->telegramService->queueMessageForUsers(
            $this->getTicketStaffRecipients($ticket),
            $message,
            '',
            ['reply_markup' => ['inline_keyboard' => $this->buildTicketActionButtons($ticket)]],
            ['type' => 'ticket', 'ticket_id' => (int) $ticket->id]
        );
    }

    public function handleTicketAdminReplied(array $payload): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_ticket_replied', true)) {
            return;
        }

        [$ticket, $ticketMessage] = $payload;
        $ticket = $ticket?->fresh(['user', 'node', 'assignedAdmin']);
        if (!$ticket || !($ticketMessage instanceof TicketMessage)) {
            return;
        }

        $owner = $ticket->user;
        if (!$owner || empty($owner->telegram_id)) {
            return;
        }

        $message = $this->buildTicketNoticeMessage($ticket, '工单有新的处理回复', $ticketMessage, false);
        if ($this->shouldAllowUserTicketActions($owner, $ticket)) {
            $this->telegramService->queueMessage((int) $owner->telegram_id, $message, '', [
                'reply_markup' => ['inline_keyboard' => $this->buildTicketActionButtons($ticket)],
            ], ['type' => 'ticket', 'ticket_id' => (int) $ticket->id]);
            return;
        }

        $this->telegramService->queueMessage((int) $owner->telegram_id, $message, '');
    }

    public function handleTicketClosed(array $payload): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_ticket_closed', true)) {
            return;
        }

        $ticket = $payload['ticket'] ?? null;
        $closedByRole = (string) ($payload['closed_by_role'] ?? 'admin');
        if (!($ticket instanceof Ticket)) {
            return;
        }

        $ticket = $ticket->fresh(['user', 'node', 'assignedAdmin']);
        if (!$ticket) {
            return;
        }

        $message = "工单 #{$ticket->id} 已关闭。\n主题：{$ticket->subject}\n关闭方：{$this->humanizeTicketActorRole($closedByRole)}";
        if ($closedByRole === 'user') {
            $this->telegramService->queueMessageForUsers($this->getTicketStaffRecipients($ticket), $message);
            return;
        }

        $owner = $ticket->user;
        if ($owner && !empty($owner->telegram_id)) {
            $this->telegramService->queueMessage((int) $owner->telegram_id, $message);
        }
    }

    public function handlePaymentNotify(mixed $order): void
    {
        if (!($order instanceof Order)) {
            return;
        }

        if (!$this->isNotifyEnabled('telegram_notify_payment_success', true)) {
            return;
        }

        $order->loadMissing(['payment', 'user', 'plan']);
        $payment = $order->payment;
        $user = $order->user;

        $message = "支付成功\n";
        $message .= "订单号：{$order->trade_no}\n";
        $message .= "金额：" . number_format(((int) $order->total_amount) / 100, 2, '.', '') . " 元\n";
        $message .= "套餐：" . ($order->plan?->name ?: '-') . "\n";
        $message .= "支付渠道：" . ($payment?->name ?: '-');

        if ($user && !empty($user->telegram_id)) {
            $this->telegramService->queueMessage((int) $user->telegram_id, $message);
        }

        $adminMessage = "用户支付成功\n用户：" . ($user?->email ?: '-') . "\n" . $message;
        $this->telegramService->sendMessageWithAdmin($adminMessage, true, '');
    }

    public function handleNoticePublished(mixed $payload): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_notice_published', true)) {
            return;
        }

        $notice = $payload['notice'] ?? $payload;
        if (!($notice instanceof Notice) || !(bool) $notice->show) {
            return;
        }

        $message = "公告发布\n标题：{$notice->title}\n";
        $message .= "标签：" . implode(' / ', array_filter((array) ($notice->tags ?? []))) . "\n";
        $message .= "内容摘要：" . $this->limitText((string) $notice->content, 180);

        $this->telegramService->queueMessageForUsers($this->getNoticeRecipients($notice), $message, '');
    }

    public function handleTcpingAlertTriggered(array $payload): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_tcping_alert', true)) {
            return;
        }

        $node = $payload['node'] ?? null;
        $alert = $payload['alert'] ?? null;
        if (!$node) {
            return;
        }

        $message = "TCPing 告警\n节点：{$node->name}\n协议：{$node->protocol}\n状态：离线\n错误：" . ($alert?->latest_error ?: $node->tcping_last_error ?: '-') . "\n";
        $message .= "触发时间：" . date('Y-m-d H:i:s', (int) ($alert?->triggered_at ?: time()));

        $this->telegramService->queueMessageForUsers($this->getNodeAlertRecipients($node), $message, '');
    }

    public function handleTcpingAlertRecovered(array $payload): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_tcping_recover', true)) {
            return;
        }

        $node = $payload['node'] ?? null;
        $alert = $payload['alert'] ?? null;
        if (!$node) {
            return;
        }

        $message = "TCPing 恢复\n节点：{$node->name}\n协议：{$node->protocol}\n状态：已恢复\n";
        $message .= "恢复时间：" . date('Y-m-d H:i:s', (int) ($alert?->recovered_at ?: time()));

        $this->telegramService->queueMessageForUsers($this->getNodeAlertRecipients($node), $message, '');
    }

    public function handleRefundRequestCreated(OrderRefundRequest $request): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_refund_status', true)) {
            return;
        }

        $request->loadMissing(['user', 'plan', 'assignedAdmin']);
        $message = "退款申请已创建\n申请单：#{$request->id}\n用户：{$request->user?->email}\n套餐：{$request->plan?->name}\n状态：待处理";
        $this->telegramService->queueMessageForUsers($this->getRefundAdminRecipients($request), $message, '');
    }

    public function handleRefundVoteStarted(OrderRefundRequest $request): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_refund_vote', true)) {
            return;
        }

        $request->loadMissing(['user', 'plan', 'assignedAdmin']);
        $message = "退款争议投票已开启\n申请单：#{$request->id}\n用户：{$request->user?->email}\n套餐：{$request->plan?->name}\n截止时间：" . ($request->voting_ends_at?->format('Y-m-d H:i:s') ?: '-');

        $recipients = $this->mergeRecipients($this->getRefundAdminRecipients($request), [$request->user]);
        $this->telegramService->queueMessageForUsers($recipients, $message, '');
    }

    public function handleRefundVoteCast(array $payload): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_refund_vote', true)) {
            return;
        }

        $request = $payload['request'] ?? null;
        $voter = $payload['voter'] ?? null;
        $vote = (string) ($payload['vote'] ?? '');
        if (!($request instanceof OrderRefundRequest)) {
            return;
        }

        $request->loadMissing(['user', 'plan', 'assignedAdmin']);
        $message = "退款争议有新投票\n申请单：#{$request->id}\n用户：{$request->user?->email}\n投票人：" . ($voter?->email ?: '-') . "\n结果：" . ($vote === 'approve' ? '支持退款' : '反对退款');
        $this->telegramService->queueMessageForUsers($this->getRefundAdminRecipients($request), $message, '');
    }

    public function handleRefundStatusChanged(OrderRefundRequest $request): void
    {
        if (!$this->isNotifyEnabled('telegram_notify_refund_status', true)) {
            return;
        }

        $request->loadMissing(['user', 'plan', 'assignedAdmin']);
        $message = "退款状态更新\n申请单：#{$request->id}\n用户：{$request->user?->email}\n套餐：{$request->plan?->name}\n状态：{$this->humanizeRefundStatus((string) $request->status)}";
        if (!empty($request->reason)) {
            $message .= "\n说明：" . $this->limitText((string) $request->reason, 180);
        }

        $recipients = $this->mergeRecipients($this->getRefundAdminRecipients($request), [$request->user]);
        $this->telegramService->queueMessageForUsers($recipients, $message, '');
    }

    public function addBotCommands(array $commands): array
    {
        foreach ($this->commandConfigs as $command => $config) {
            $commands[] = [
                'command' => ltrim($command, '/'),
                'description' => $config['description'],
            ];
        }

        return $commands;
    }

    protected function sendMessage(object $msg, string $message, array $options = [], string $parseMode = ''): void
    {
        $this->telegramService->sendMessage((int) $msg->chat_id, $message, $parseMode, $options);
    }

    protected function checkPrivateChat(object $msg): bool
    {
        if (!$msg->is_private) {
            $this->sendMessage($msg, '请在私聊中使用此命令。');
            return false;
        }

        return true;
    }

    protected function getBoundUser(object $msg): ?User
    {
        $user = User::query()->where('telegram_id', $msg->chat_id)->first();
        if (!$user) {
            $this->sendMessage($msg, '请先绑定账号。');
            return null;
        }

        return $user;
    }

    protected function handleTicketReplyByContext(object $msg, array $context): void
    {
        $user = $this->getBoundUser($msg);
        if (!$user) {
            return;
        }

        $ticketId = (int) ($context['ticket_id'] ?? 0);
        if ($ticketId <= 0) {
            $this->sendMessage($msg, '未找到对应工单。');
            return;
        }

        $ticket = Ticket::query()->with(['user', 'node', 'assignedAdmin'])->find($ticketId);
        if (!$ticket) {
            $this->sendMessage($msg, '工单不存在。');
            return;
        }

        $this->assertTicketAccess($ticket, $user);
        $this->ticketService->replyByActor($ticket, (string) $msg->text, $user);
        $this->sendMessage($msg, "工单 #{$ticketId} 回复成功。");
        $this->sendTicketDetailMessage($msg, $ticket->fresh(['user', 'node', 'assignedAdmin']), $user, '最新回复已提交。');
    }

    protected function sendTicketListMessage(object $msg, User $user, string $scope): void
    {
        $tickets = $this->listTicketsForActor($user, $scope);
        if ($tickets->isEmpty()) {
            $this->sendMessage($msg, "当前视图暂无工单。\n视图：{$this->humanizeTicketListScope($scope)}");
            return;
        }

        $lines = [
            '工单列表 - ' . $this->humanizeTicketListScope($scope),
            '',
        ];

        foreach ($tickets as $ticket) {
            $lines[] = sprintf(
                '#%d [%s] %s',
                $ticket->id,
                $ticket->status === Ticket::STATUS_CLOSED ? '已关闭' : '处理中',
                $this->limitText((string) $ticket->subject, 48)
            );
        }

        $buttons = [];
        foreach ($tickets as $ticket) {
            $buttons[] = [[
                'text' => '#' . $ticket->id . ' ' . $this->limitText((string) $ticket->subject, 18),
                'callback_data' => 'tk:view:' . $ticket->id,
            ]];
        }

        $this->telegramService->sendMessage($msg->chat_id, implode("\n", $lines), '', [
            'reply_markup' => ['inline_keyboard' => array_merge($buttons, $this->buildTicketMenuButtons($user))],
        ]);
    }

    protected function sendTicketDetailMessage(object $msg, Ticket $ticket, User $actor, ?string $prefix = null): void
    {
        $this->assertTicketAccess($ticket, $actor);

        $ticket->loadMissing(['user', 'node', 'assignedAdmin', 'messages.user']);
        $messages = $ticket->messages()->orderByDesc('id')->limit(6)->get()->reverse();
        $lines = [];
        if ($prefix) {
            $lines[] = trim($prefix);
            $lines[] = '';
        }

        $lines[] = "工单 #{$ticket->id}";
        $lines[] = "主题：{$ticket->subject}";
        $lines[] = "状态：" . ($ticket->status === Ticket::STATUS_CLOSED ? '已关闭' : '处理中');
        $lines[] = "用户：" . ($ticket->user?->email ?: '-');
        $lines[] = "节点：" . ($ticket->node?->name ?: '-');
        $lines[] = "负责人：" . ($ticket->assignedAdmin?->email ?: '未指定');
        $lines[] = '';
        $lines[] = '最近消息：';

        /** @var TicketMessage $message */
        foreach ($messages as $message) {
            $lines[] = sprintf(
                '[%s] %s',
                $this->resolveTicketMessageAuthorLabel($ticket, $message),
                $this->limitText((string) $message->message, 140)
            );
        }

        if ($ticket->status !== Ticket::STATUS_CLOSED) {
            $lines[] = '';
            $lines[] = '直接回复此消息即可继续处理工单。';
        }

        $this->telegramService->sendContextMessage((int) $msg->chat_id, implode("\n", $lines), [
            'type' => 'ticket',
            'ticket_id' => (int) $ticket->id,
        ], '', [
            'reply_markup' => ['inline_keyboard' => $this->buildTicketActionButtons($ticket)],
        ]);
    }

    protected function assertTicketAccess(Ticket $ticket, User $actor): void
    {
        if ((int) $actor->id === (int) $ticket->user_id) {
            if (!$this->shouldAllowUserTicketActions($actor, $ticket)) {
                throw new \RuntimeException('当前未开启 Telegram 用户工单处理。');
            }
            return;
        }

        if ($this->ticketService->canManageByAdmin($actor, $ticket)) {
            return;
        }

        throw new \RuntimeException('无权访问该工单。');
    }

    protected function shouldAllowUserTicketActions(User $user, ?Ticket $ticket = null): bool
    {
        if ($this->hasAdministrativeTicketAccess($user)) {
            return true;
        }

        if ($ticket && (int) ($ticket->assigned_admin_user_id ?? 0) === (int) $user->id) {
            return true;
        }

        return (bool) admin_setting('telegram_user_ticket_enable', 1);
    }

    protected function hasAdministrativeTicketAccess(User $user): bool
    {
        return (bool) ($user->is_admin ?? false) || (bool) ($user->is_super_admin ?? false) || (bool) ($user->is_staff ?? false);
    }

    protected function listTicketsForActor(User $user, string $scope): Collection
    {
        $scope = strtolower(trim($scope));
        if ($scope === 'admin' && $this->hasAdministrativeTicketAccess($user)) {
            return Ticket::query()
                ->with(['user', 'node', 'assignedAdmin'])
                ->orderBy('status')
                ->orderByDesc('updated_at')
                ->limit(8)
                ->get();
        }

        if ($scope === 'assigned') {
            return Ticket::query()
                ->with(['user', 'node', 'assignedAdmin'])
                ->where('assigned_admin_user_id', $user->id)
                ->orderBy('status')
                ->orderByDesc('updated_at')
                ->limit(8)
                ->get();
        }

        return Ticket::query()
            ->with(['user', 'node', 'assignedAdmin'])
            ->where('user_id', $user->id)
            ->orderByDesc('updated_at')
            ->limit(8)
            ->get();
    }

    protected function buildTicketMenuButtons(User $user): array
    {
        $buttons = [];

        if ($this->shouldAllowUserTicketActions($user)) {
            $buttons[] = [[
                'text' => '我的工单',
                'callback_data' => 'tk:list:mine',
            ]];
        }

        if (Ticket::query()->where('assigned_admin_user_id', $user->id)->exists()) {
            $buttons[] = [[
                'text' => '指派给我',
                'callback_data' => 'tk:list:assigned',
            ]];
        }

        if ($this->hasAdministrativeTicketAccess($user)) {
            $buttons[] = [[
                'text' => '后台待处理',
                'callback_data' => 'tk:list:admin',
            ]];
        }

        return $buttons;
    }

    protected function buildTicketActionButtons(Ticket $ticket): array
    {
        $buttons = [[
            ['text' => '刷新详情', 'callback_data' => 'tk:view:' . $ticket->id],
        ]];

        if ((int) $ticket->status !== Ticket::STATUS_CLOSED) {
            $buttons[] = [[
                'text' => '关闭工单',
                'callback_data' => 'tk:close:' . $ticket->id,
            ]];
        }

        return $buttons;
    }

    protected function buildTicketNoticeMessage(Ticket $ticket, string $title, ?TicketMessage $message, bool $forStaff): string
    {
        $text = $title . "\n";
        $text .= "工单：#{$ticket->id}\n";
        $text .= "主题：{$ticket->subject}\n";
        $text .= "状态：" . ($ticket->status === Ticket::STATUS_CLOSED ? '已关闭' : '处理中') . "\n";
        $text .= "用户：" . ($ticket->user?->email ?: '-') . "\n";
        $text .= "节点：" . ($ticket->node?->name ?: '-') . "\n";
        if ($forStaff) {
            $text .= "负责人：" . ($ticket->assignedAdmin?->email ?: '未指定') . "\n";
        }

        if ($message) {
            $text .= "最新内容：" . $this->limitText((string) $message->message, 160) . "\n";
        }

        $text .= $forStaff ? '直接回复此消息即可处理工单。' : '直接回复此消息即可继续沟通。';
        return $text;
    }

    protected function resolveTicketMessageAuthorLabel(Ticket $ticket, TicketMessage $message): string
    {
        if ((int) $message->user_id === (int) $ticket->user_id) {
            return '用户';
        }

        if ((int) ($ticket->assigned_admin_user_id ?? 0) === (int) $message->user_id) {
            return '节点管理员';
        }

        return '管理员';
    }

    protected function getTicketStaffRecipients(Ticket $ticket): Collection
    {
        $assigned = null;
        if ((int) ($ticket->assigned_admin_user_id ?? 0) > 0) {
            $assigned = User::query()->find((int) $ticket->assigned_admin_user_id);
        }

        return $this->mergeRecipients($this->getStaffRecipients(true), [$assigned]);
    }

    protected function getNodeAlertRecipients(object $node): Collection
    {
        $owner = null;
        if ((int) ($node->user_id ?? 0) > 0) {
            $owner = User::query()->find((int) $node->user_id);
        }

        return $this->mergeRecipients($this->getStaffRecipients(true), [$owner]);
    }

    protected function getRefundAdminRecipients(OrderRefundRequest $request): Collection
    {
        $assigned = null;
        if ((int) ($request->assigned_admin_user_id ?? 0) > 0) {
            $assigned = User::query()->find((int) $request->assigned_admin_user_id);
        }

        return $this->mergeRecipients($this->getStaffRecipients(true), [$assigned]);
    }

    protected function getNoticeRecipients(Notice $notice): Collection
    {
        if ((string) ($notice->scope_type ?? Notice::SCOPE_GLOBAL) === Notice::SCOPE_GLOBAL) {
            return User::query()
                ->whereNotNull('telegram_id')
                ->get();
        }

        $planIds = collect($notice->target_plan_ids ?? [])
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->values()
            ->all();

        if (empty($planIds)) {
            return collect();
        }

        $userIds = UserPlanSubscription::query()
            ->whereIn('plan_id', $planIds)
            ->active(time())
            ->pluck('user_id')
            ->map(fn ($value) => (int) $value)
            ->unique()
            ->values()
            ->all();

        if (empty($userIds)) {
            return collect();
        }

        return User::query()
            ->whereIn('id', $userIds)
            ->whereNotNull('telegram_id')
            ->get();
    }

    protected function getStaffRecipients(bool $includeStaff = true): Collection
    {
        return User::query()
            ->whereNotNull('telegram_id')
            ->where(function ($query) use ($includeStaff) {
                $query->where('is_admin', 1)
                    ->orWhere('is_super_admin', 1);

                if ($includeStaff) {
                    $query->orWhere('is_staff', 1);
                }
            })
            ->get();
    }

    protected function mergeRecipients(iterable ...$groups): Collection
    {
        return collect($groups)
            ->flatten(1)
            ->filter(fn ($user) => $user instanceof User && !empty($user->telegram_id))
            ->unique(fn (User $user) => (int) $user->telegram_id)
            ->values();
    }

    protected function isNotifyEnabled(string $key, bool $default = true): bool
    {
        return (bool) admin_setting($key, $default ? 1 : 0);
    }

    protected function extractTokenFromUrl(string $url): ?string
    {
        $parsedUrl = parse_url($url);

        if (isset($parsedUrl['query'])) {
            parse_str($parsedUrl['query'], $query);
            if (isset($query['token'])) {
                return $query['token'];
            }
        }

        if (isset($parsedUrl['path'])) {
            $pathParts = explode('/', trim($parsedUrl['path'], '/'));
            $lastPart = end($pathParts);
            return $lastPart ?: null;
        }

        return null;
    }

    protected function humanizeTicketListScope(string $scope): string
    {
        return match (strtolower(trim($scope))) {
            'assigned' => '指派给我',
            'admin' => '后台待处理',
            default => '我的工单',
        };
    }

    protected function humanizeTicketActorRole(string $role): string
    {
        return match ($role) {
            'user' => '用户',
            'assigned_admin' => '节点管理员',
            default => '管理员',
        };
    }

    protected function humanizeRefundStatus(string $status): string
    {
        return match ($status) {
            OrderRefundRequest::STATUS_PENDING => '待处理',
            OrderRefundRequest::STATUS_VOTING => '投票中',
            OrderRefundRequest::STATUS_APPROVED => '已批准',
            OrderRefundRequest::STATUS_DENIED => '已拒绝',
            OrderRefundRequest::STATUS_REFUNDED => '已退款',
            OrderRefundRequest::STATUS_FAILED => '退款失败',
            default => $status,
        };
    }

    protected function limitText(string $text, int $limit = 120): string
    {
        $text = trim(preg_replace("/\s+/u", ' ', $text) ?: $text);
        if (mb_strlen($text) <= $limit) {
            return $text;
        }

        return mb_substr($text, 0, $limit - 1) . '…';
    }

    private function transferToGBString(float $transfer, int $decimals = 2): string
    {
        return number_format(Helper::transferToGB($transfer), $decimals, '.', '');
    }
}
