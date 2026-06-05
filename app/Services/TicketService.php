<?php

namespace App\Services;

use App\Exceptions\ApiException;
use App\Models\ServerNode;
use App\Models\Ticket;
use App\Models\TicketMessage;
use App\Models\User;
use App\Services\Plugin\HookManager;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;

class TicketService
{
    public function reply($ticket, $message, $userId)
    {
        try {
            return $this->replyInternal($ticket, (string) $message, (int) $userId, false, true);
        } catch (\Throwable $e) {
            return false;
        }
    }

    public function replyByAdmin($ticketId, $message, $userId): void
    {
        $ticket = Ticket::query()->where('id', $ticketId)->first();
        if (!$ticket) {
            throw new ApiException('工单不存在');
        }

        $ticketMessage = $this->replyInternal($ticket, (string) $message, (int) $userId, true, false);
        HookManager::call('ticket.reply.admin.after', [$ticket, $ticketMessage]);
        $this->sendEmailNotify($ticket, $ticketMessage);
    }

    public function replyByActor(Ticket $ticket, string $message, User $actor): TicketMessage
    {
        if ((int) $actor->id === (int) $ticket->user_id) {
            $ticketMessage = $this->replyInternal($ticket, $message, (int) $actor->id, false, true);
            HookManager::call('ticket.reply.user.after', $ticket);

            return $ticketMessage;
        }

        if (!$this->canManageByAdmin($actor, $ticket)) {
            throw new ApiException('Permission denied');
        }

        $ticketMessage = $this->replyInternal($ticket, $message, (int) $actor->id, true, false);
        HookManager::call('ticket.reply.admin.after', [$ticket, $ticketMessage]);
        $this->sendEmailNotify($ticket, $ticketMessage);

        return $ticketMessage;
    }

    public function closeByActor(Ticket $ticket, User $actor): Ticket
    {
        if ((int) $actor->id !== (int) $ticket->user_id && !$this->canManageByAdmin($actor, $ticket)) {
            throw new ApiException('Permission denied');
        }

        $ticket = DB::transaction(function () use ($ticket) {
            $lockedTicket = Ticket::query()->lockForUpdate()->find($ticket->id);
            if (!$lockedTicket) {
                throw new ApiException('工单不存在');
            }
            if ((int) $lockedTicket->status === Ticket::STATUS_CLOSED) {
                return $lockedTicket;
            }
            $lockedTicket->status = Ticket::STATUS_CLOSED;
            if (!$lockedTicket->save()) {
                throw new ApiException('关闭失败');
            }
            return $lockedTicket;
        });

        HookManager::call('ticket.close.after', [
            'ticket' => $ticket->fresh(['user', 'node', 'assignedAdmin']),
            'closed_by_user_id' => (int) $actor->id,
            'closed_by_role' => $this->resolveActorRole($actor, $ticket),
        ]);

        return $ticket;
    }

    public function canManageByAdmin(User $actor, Ticket $ticket): bool
    {
        if ((bool) ($actor->is_admin ?? false) || (bool) ($actor->is_super_admin ?? false) || (bool) ($actor->is_staff ?? false)) {
            return true;
        }

        return (int) ($ticket->assigned_admin_user_id ?? 0) === (int) $actor->id;
    }

    public function createTicket($userId, $subject, $level, $message, ?int $nodeId = null)
    {
        [$ticket, $ticketMessage, $assignedAdminUserId] = DB::transaction(function () use ($userId, $subject, $level, $message, $nodeId) {
            User::query()->whereKey($userId)->lockForUpdate()->first();

            $existing = Ticket::query()->where('status', Ticket::STATUS_OPENING)
                ->where('user_id', $userId)
                ->when($nodeId !== null, fn ($q) => $q->where('node_id', $nodeId), fn ($q) => $q->whereNull('node_id'))
                ->lockForUpdate()
                ->first();
            if ($existing) {
                throw new ApiException('存在未关闭的工单');
            }

            $assignedAdminUserId = null;
            if ($nodeId !== null) {
                $node = ServerNode::query()->whereKey($nodeId)->first();
                $assignedAdminUserId = $node?->user_id;
            }

            $ticket = Ticket::query()->create([
                'user_id' => $userId,
                'node_id' => $nodeId,
                'assigned_admin_user_id' => $assignedAdminUserId,
                'subject' => $subject,
                'level' => $level,
                'status' => Ticket::STATUS_OPENING,
                'last_reply_user_id' => $userId,
                'reply_status' => Ticket::STATUS_CLOSED,
            ]);
            if (!$ticket) {
                throw new ApiException('工单创建失败');
            }

            $ticketMessage = TicketMessage::query()->create([
                'user_id' => $userId,
                'ticket_id' => $ticket->id,
                'message' => $message
            ]);
            if (!$ticketMessage) {
                throw new ApiException('工单消息创建失败');
            }

            return [$ticket, $ticketMessage, (int) ($assignedAdminUserId ?? 0)];
        });

        if ($assignedAdminUserId > 0) {
            $this->sendEmailNotifyAdmin($ticket, $ticketMessage, $assignedAdminUserId);
        }

        return $ticket;
    }

    private function replyInternal(Ticket $ticket, string $message, int $userId, bool $isAdminReply, bool $enforceUserTurn): TicketMessage
    {
        $message = trim($message);
        if ($message === '') {
            throw new ApiException('消息不能为空');
        }

        return DB::transaction(function () use ($ticket, $message, $userId, $isAdminReply, $enforceUserTurn) {
            $lockedTicket = Ticket::query()->lockForUpdate()->find($ticket->id);
            if (!$lockedTicket) {
                throw new ApiException('工单不存在');
            }
            if ((int) $lockedTicket->status === Ticket::STATUS_CLOSED) {
                throw new ApiException('工单已关闭，无法继续回复');
            }

            if ($enforceUserTurn) {
                $lastMessage = $this->getLastMessage((int) $lockedTicket->id, true);
                if ($lastMessage && (int) $lastMessage->user_id === $userId) {
                    throw new ApiException('请等待技术人员回复');
                }
            }

            $ticketMessage = TicketMessage::query()->create([
                'user_id' => $userId,
                'ticket_id' => $lockedTicket->id,
                'message' => $message,
            ]);

            if (!$ticketMessage) {
                throw new ApiException('工单回复失败');
            }

            $lockedTicket->status = Ticket::STATUS_OPENING;
            $lockedTicket->reply_status = $isAdminReply ? Ticket::STATUS_OPENING : Ticket::STATUS_CLOSED;
            $lockedTicket->last_reply_user_id = $userId;

            if (!$lockedTicket->save()) {
                throw new ApiException('工单回复失败');
            }

            $ticket->forceFill($lockedTicket->getAttributes());
            return $ticketMessage;
        });
    }

    private function resolveActorRole(User $actor, Ticket $ticket): string
    {
        if ((int) $actor->id === (int) $ticket->user_id) {
            return 'user';
        }

        if ((int) ($ticket->assigned_admin_user_id ?? 0) === (int) $actor->id) {
            return 'assigned_admin';
        }

        return 'admin';
    }

    private function getLastMessage(int $ticketId, bool $forUpdate = false): ?TicketMessage
    {
        $query = TicketMessage::query()
            ->where('ticket_id', $ticketId)
            ->orderByDesc('id');
        if ($forUpdate) {
            $query->lockForUpdate();
        }

        return $query->first();
    }

    private function sendEmailNotifyAdmin(Ticket $ticket, TicketMessage $ticketMessage, int $adminUserId): void
    {
        if (config('ops.telegram_only_mode')) {
            return;
        }

        $admin = User::find($adminUserId);
        if (!$admin) {
            return;
        }

        $cacheKey = 'ticket_notify_admin_' . $adminUserId . '_' . ($ticket->node_id ?? 0);
        if (Cache::get($cacheKey)) {
            return;
        }
        Cache::put($cacheKey, 1, 600);

        MailService::dispatchEmail([
            'email' => $admin->email,
            'subject' => '节点工单通知 - ' . admin_setting('app_name', 'Portal'),
            'template_name' => 'notify',
            'template_value' => [
                'name' => admin_setting('app_name', 'Portal'),
                'url' => admin_setting('app_url'),
                'content' => "节点ID：{$ticket->node_id}\r\n主题：{$ticket->subject}\r\n内容：{$ticketMessage->message}"
            ]
        ]);
    }

    private function sendEmailNotify(Ticket $ticket, TicketMessage $ticketMessage): void
    {
        if (config('ops.telegram_only_mode')) {
            return;
        }

        $user = User::find($ticket->user_id);
        if (!$user) {
            return;
        }

        $cacheKey = 'ticket_sendEmailNotify_' . $ticket->user_id;
        if (!Cache::get($cacheKey)) {
            Cache::put($cacheKey, 1, 1800);
            MailService::dispatchEmail([
                'email' => $user->email,
                'subject' => '您在' . admin_setting('app_name', 'Portal') . '的工单得到了回复',
                'template_name' => 'notify',
                'template_value' => [
                    'name' => admin_setting('app_name', 'Portal'),
                    'url' => admin_setting('app_url'),
                    'content' => "主题：{$ticket->subject}\r\n回复内容：{$ticketMessage->message}"
                ]
            ]);
        }
    }
}
