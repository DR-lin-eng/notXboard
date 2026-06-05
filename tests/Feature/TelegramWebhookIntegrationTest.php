<?php

namespace Tests\Feature;

use App\Models\Plugin;
use App\Models\Ticket;
use App\Models\TicketMessage;
use App\Models\User;
use App\Services\TelegramMessageContextService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Http;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class TelegramWebhookIntegrationTest extends TestCase
{
    use RefreshDatabase;

    private function enableTelegramPlugin(): void
    {
        Plugin::query()->create([
            'name' => 'Telegram',
            'code' => 'telegram',
            'type' => Plugin::TYPE_FEATURE,
            'version' => '1.0.1',
            'is_enabled' => true,
            'config' => json_encode([]),
        ]);

        admin_setting([
            'telegram_bot_enable' => 1,
            'telegram_bot_token' => 'test-bot-token',
            'telegram_user_ticket_enable' => 1,
            'telegram_notify_ticket_created' => 0,
            'telegram_notify_ticket_replied' => 0,
            'telegram_notify_ticket_closed' => 0,
            'telegram_notify_payment_success' => 0,
            'telegram_notify_notice_published' => 0,
            'telegram_notify_tcping_alert' => 0,
            'telegram_notify_tcping_recover' => 0,
            'telegram_notify_refund_vote' => 0,
            'telegram_notify_refund_status' => 0,
        ]);

        Http::fake([
            'https://api.telegram.org/*' => Http::response([
                'ok' => true,
                'result' => [
                    'message_id' => 501,
                    'username' => 'test_bot',
                ],
            ], 200),
        ]);
    }

    private function webhookPath(): string
    {
        return '/api/v1/guest/telegram/webhook?access_token=' . md5('test-bot-token');
    }

    private function createTicketThread(User $owner, User $admin): Ticket
    {
        $ticket = Ticket::query()->create([
            'user_id' => $owner->id,
            'assigned_admin_user_id' => $admin->id,
            'subject' => '节点异常',
            'level' => '1',
            'status' => Ticket::STATUS_OPENING,
            'reply_status' => Ticket::STATUS_OPENING,
            'last_reply_user_id' => $admin->id,
        ]);

        TicketMessage::query()->create([
            'ticket_id' => $ticket->id,
            'user_id' => $owner->id,
            'message' => '最初工单',
        ]);

        TicketMessage::query()->create([
            'ticket_id' => $ticket->id,
            'user_id' => $admin->id,
            'message' => '请补充截图',
        ]);

        return $ticket;
    }

    private function telegramReplyPayload(int $chatId, int $replyMessageId, string $text, int $messageId = 10001): array
    {
        return [
            'message' => [
                'message_id' => $messageId,
                'from' => ['id' => $chatId],
                'chat' => ['id' => $chatId, 'type' => 'private'],
                'text' => $text,
                'reply_to_message' => [
                    'message_id' => $replyMessageId,
                    'text' => '工单 #' . $replyMessageId,
                ],
            ],
        ];
    }

    private function v2Path(string $endpoint): string
    {
        $securePath = (string) admin_setting('secure_path', admin_setting('frontend_admin_path', hash('crc32b', config('app.key'))));
        return '/api/v2/' . $securePath . '/' . ltrim($endpoint, '/');
    }

    public function test_bound_user_can_reply_to_own_ticket_via_telegram_reply_context(): void
    {
        $this->enableTelegramPlugin();

        $owner = User::factory()->create([
            'telegram_id' => 123456,
        ]);
        $admin = User::factory()->create([
            'is_admin' => true,
        ]);

        $ticket = $this->createTicketThread($owner, $admin);
        app(TelegramMessageContextService::class)->put((int) $owner->telegram_id, 9001, [
            'type' => 'ticket',
            'ticket_id' => $ticket->id,
        ]);

        $response = $this->postJson($this->webhookPath(), $this->telegramReplyPayload((int) $owner->telegram_id, 9001, '这是 Telegram 追加回复'));

        $response->assertOk();
        $this->assertDatabaseHas('v2_ticket_message', [
            'ticket_id' => $ticket->id,
            'user_id' => $owner->id,
            'message' => '这是 Telegram 追加回复',
        ]);
        $this->assertDatabaseHas('v2_ticket', [
            'id' => $ticket->id,
            'last_reply_user_id' => $owner->id,
        ]);
    }

    public function test_unrelated_bound_user_cannot_reply_to_someone_else_ticket_via_telegram(): void
    {
        $this->enableTelegramPlugin();

        $owner = User::factory()->create([
            'telegram_id' => 123456,
        ]);
        $admin = User::factory()->create([
            'is_admin' => true,
        ]);
        $intruder = User::factory()->create([
            'telegram_id' => 789012,
        ]);

        $ticket = $this->createTicketThread($owner, $admin);
        app(TelegramMessageContextService::class)->put((int) $intruder->telegram_id, 9002, [
            'type' => 'ticket',
            'ticket_id' => $ticket->id,
        ]);

        $beforeCount = TicketMessage::query()->where('ticket_id', $ticket->id)->count();
        $response = $this->postJson($this->webhookPath(), $this->telegramReplyPayload((int) $intruder->telegram_id, 9002, '越权回复'));

        $response->assertOk();
        $this->assertSame($beforeCount, TicketMessage::query()->where('ticket_id', $ticket->id)->count());
        $this->assertDatabaseMissing('v2_ticket_message', [
            'ticket_id' => $ticket->id,
            'user_id' => $intruder->id,
            'message' => '越权回复',
        ]);
    }

    public function test_super_admin_config_fetch_exposes_telegram_settings(): void
    {
        admin_setting([
            'telegram_bot_enable' => 1,
            'telegram_bot_token' => 'abc',
            'telegram_discuss_link' => 'https://t.me/example',
            'telegram_user_ticket_enable' => 1,
            'telegram_notify_ticket_created' => 1,
            'telegram_notify_ticket_replied' => 0,
            'telegram_notify_ticket_closed' => 1,
            'telegram_notify_payment_success' => 1,
            'telegram_notify_notice_published' => 0,
            'telegram_notify_tcping_alert' => 1,
            'telegram_notify_tcping_recover' => 1,
            'telegram_notify_refund_vote' => 0,
            'telegram_notify_refund_status' => 1,
        ]);

        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => true,
        ]);
        Sanctum::actingAs($admin);

        $this->getJson($this->v2Path('config/fetch?key=telegram'))
            ->assertOk()
            ->assertJsonPath('data.telegram.telegram_bot_enable', true)
            ->assertJsonPath('data.telegram.telegram_bot_token', 'abc')
            ->assertJsonPath('data.telegram.telegram_user_ticket_enable', true)
            ->assertJsonPath('data.telegram.telegram_notify_ticket_replied', false)
            ->assertJsonPath('data.telegram.telegram_notify_tcping_alert', true)
            ->assertJsonPath('data.telegram.telegram_notify_refund_vote', false);
    }
}
