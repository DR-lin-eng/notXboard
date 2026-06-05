<?php

namespace App\Services;

use App\Models\User;
use App\Models\UserBanRecord;
use App\Services\Plugin\HookManager;
use Illuminate\Support\Facades\DB;
use Throwable;

class UserBanService
{
    public function banUser(User $user, ?User $admin, string $reason, array $context = []): UserBanRecord
    {
        $reason = trim($reason);
        if ($reason === '') {
            throw new \InvalidArgumentException('封禁原因不能为空');
        }

        $timestamp = time();

        /** @var UserBanRecord $record */
        $record = DB::transaction(function () use ($user, $admin, $reason, $context, $timestamp): UserBanRecord {
            $user->forceFill([
                'banned' => 1,
                'ban_reason' => $reason,
                'banned_at' => $timestamp,
                'banned_by_admin_id' => $admin?->id,
            ])->saveOrFail();

            $record = UserBanRecord::query()->create([
                'user_id' => $user->id,
                'admin_id' => $admin?->id,
                'action' => UserBanRecord::ACTION_BAN,
                'reason' => $reason,
                'source' => (string) ($context['source'] ?? 'manual'),
                'context' => $context ?: null,
                'created_at' => $timestamp,
                'updated_at' => $timestamp,
            ]);

            app(AuthService::class, ['user' => $user])->removeAllSessions();

            return $record;
        });

        HookManager::call('user.banned', [
            'user' => $user->fresh(),
            'admin' => $admin,
            'reason' => $reason,
            'record' => $record,
            'context' => $context,
        ]);

        $this->notifySuperAdminsAboutBan($user->fresh(), $admin, $reason, $context);

        return $record;
    }

    public function unbanUser(User $user, ?User $admin = null, ?string $reason = null, array $context = []): UserBanRecord
    {
        $timestamp = time();
        $reason = trim((string) $reason);
        if ($reason === '') {
            $reason = 'manual unban';
        }

        /** @var UserBanRecord $record */
        $record = DB::transaction(function () use ($user, $admin, $reason, $context, $timestamp): UserBanRecord {
            $previousBanReason = $user->ban_reason;

            $user->forceFill([
                'banned' => 0,
                'ban_reason' => null,
                'banned_at' => null,
                'banned_by_admin_id' => null,
            ])->saveOrFail();

            return UserBanRecord::query()->create([
                'user_id' => $user->id,
                'admin_id' => $admin?->id,
                'action' => UserBanRecord::ACTION_UNBAN,
                'reason' => $reason,
                'source' => (string) ($context['source'] ?? 'manual'),
                'context' => array_filter(array_merge($context, [
                    'previous_ban_reason' => $previousBanReason,
                ])),
                'created_at' => $timestamp,
                'updated_at' => $timestamp,
            ]);
        });

        HookManager::call('user.unbanned', [
            'user' => $user->fresh(),
            'admin' => $admin,
            'reason' => $reason,
            'record' => $record,
            'context' => $context,
        ]);

        return $record;
    }

    private function notifySuperAdminsAboutBan(User $user, ?User $admin, string $reason, array $context = []): void
    {
        if (array_key_exists('notify', $context) && !(bool) $context['notify']) {
            return;
        }

        if (!(bool) admin_setting('telegram_bot_enable', 0) || !(bool) admin_setting('telegram_notify_user_banned', 1)) {
            return;
        }

        $recipients = User::query()
            ->where('is_super_admin', 1)
            ->whereNotNull('telegram_id')
            ->get(['id', 'email', 'telegram_id']);

        if ($recipients->isEmpty()) {
            return;
        }

        $lines = [
            '用户封禁通知',
            '用户: #' . $user->id . ' ' . $user->email,
            '操作人: ' . ($admin?->email ?: 'system'),
            '原因: ' . $reason,
        ];

        if (!empty($context['source'])) {
            $lines[] = '来源: ' . (string) $context['source'];
        }

        if (!empty($context['risk_review_id'])) {
            $lines[] = '风险审查记录: #' . (int) $context['risk_review_id'];
        }

        try {
            app(TelegramService::class)->queueMessageForUsers(
                $recipients,
                implode("\n", $lines),
                ''
            );
        } catch (Throwable) {
            // Ignore Telegram delivery failures for administrative actions.
        }
    }
}
