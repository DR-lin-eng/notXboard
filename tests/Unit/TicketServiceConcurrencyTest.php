<?php

namespace Tests\Unit;

use App\Exceptions\ApiException;
use App\Models\Ticket;
use App\Models\TicketMessage;
use App\Models\User;
use App\Services\TicketService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class TicketServiceConcurrencyTest extends TestCase
{
    use RefreshDatabase;

    public function test_user_cannot_reply_twice_without_admin_response(): void
    {
        $owner = User::factory()->create();
        $ticket = Ticket::query()->create([
            'user_id' => $owner->id,
            'subject' => '节点异常',
            'level' => 1,
            'status' => Ticket::STATUS_OPENING,
            'reply_status' => Ticket::STATUS_CLOSED,
            'last_reply_user_id' => $owner->id,
            'created_at' => time(),
            'updated_at' => time(),
        ]);
        TicketMessage::query()->create([
            'ticket_id' => $ticket->id,
            'user_id' => $owner->id,
            'message' => 'first',
            'created_at' => time(),
            'updated_at' => time(),
        ]);

        $this->expectException(ApiException::class);
        app(TicketService::class)->replyByActor($ticket, 'second', $owner);
    }

    public function test_create_ticket_blocks_second_open_ticket_for_same_scope(): void
    {
        $owner = User::factory()->create();
        $service = app(TicketService::class);

        $service->createTicket($owner->id, '问题1', 1, '内容1', null);

        $this->expectException(ApiException::class);
        $service->createTicket($owner->id, '问题2', 1, '内容2', null);
    }
}

