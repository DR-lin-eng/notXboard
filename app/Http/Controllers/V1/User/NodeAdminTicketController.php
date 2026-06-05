<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Http\Resources\TicketResource;
use App\Models\Ticket;
use App\Services\TicketService;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class NodeAdminTicketController extends Controller
{
    public function inbox(Request $request)
    {
        $request->validate([
            'status' => 'nullable|integer|in:0,1',
        ]);

        $user = Auth::user();

        $tickets = Ticket::query()
            ->where('assigned_admin_user_id', $user->id)
            ->when($request->input('status') !== null, fn ($q) => $q->where('status', (int) $request->input('status')))
            ->orderByDesc('created_at')
            ->get();

        return $this->success(TicketResource::collection($tickets));
    }

    public function detail(Request $request)
    {
        $request->validate([
            'id' => 'required|integer|min:1',
        ]);

        $user = Auth::user();

        $ticket = Ticket::query()
            ->whereKey((int) $request->input('id'))
            ->where('assigned_admin_user_id', $user->id)
            ->first()
            ?->load('message');

        if (!$ticket) {
            return $this->fail([400, __('Ticket does not exist')]);
        }

        $ticket['message'] = \App\Models\TicketMessage::where('ticket_id', $ticket->id)->get();
        $ticket['message']->each(function ($message) use ($ticket) {
            $message['is_me'] = ($message['user_id'] == $ticket->assigned_admin_user_id);
        });

        return $this->success(TicketResource::make($ticket)->additional(['message' => true]));
    }

    public function reply(Request $request)
    {
        $request->validate([
            'id' => 'required|integer|min:1',
            'message' => 'required|string',
        ]);

        $user = Auth::user();

        $ticket = Ticket::query()
            ->whereKey((int) $request->input('id'))
            ->where('assigned_admin_user_id', $user->id)
            ->first();

        if (!$ticket) {
            return $this->fail([400, __('Ticket does not exist')]);
        }

        if ($ticket->status) {
            return $this->fail([400, __('The ticket is closed and cannot be replied')]);
        }

        app(TicketService::class)->replyByAdmin($ticket->id, (string) $request->input('message'), $user->id);

        return $this->success(true);
    }

    public function close(Request $request)
    {
        $request->validate([
            'id' => 'required|integer|min:1',
        ]);

        $user = Auth::user();

        $ticket = Ticket::query()
            ->whereKey((int) $request->input('id'))
            ->where('assigned_admin_user_id', $user->id)
            ->first();

        if (!$ticket) {
            return $this->fail([400, __('Ticket does not exist')]);
        }

        try {
            app(TicketService::class)->closeByActor($ticket, $user);
        } catch (\Throwable $e) {
            return $this->fail([500, __('Close failed')]);
        }

        return $this->success(true);
    }
}
