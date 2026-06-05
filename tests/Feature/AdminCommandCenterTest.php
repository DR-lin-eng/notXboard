<?php

namespace Tests\Feature;

use App\Models\AuditLog;
use App\Models\NodeTrafficRecord;
use App\Models\Order;
use App\Models\OrderRefundRequest;
use App\Models\Plan;
use App\Models\ServerNode;
use App\Models\TcpingAgent;
use App\Models\TcpingAlert;
use App\Models\TcpingSample;
use App\Models\Ticket;
use App\Models\User;
use App\Models\UserOnlineSession;
use App\Models\UserPlanSubscription;
use App\Utils\CacheKey;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Str;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class AdminCommandCenterTest extends TestCase
{
    use RefreshDatabase;

    protected function setUp(): void
    {
        parent::setUp();
        Cache::flush();
    }

    public function test_super_admin_can_fetch_command_center_snapshot(): void
    {
        $superAdmin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => true,
            'email' => 'root@example.com',
        ]);
        $provider = User::factory()->create([
            'email' => 'provider@example.com',
        ]);
        $consumer = User::factory()->create([
            'email' => 'consumer@example.com',
            'linux_do_name' => 'Heavy User',
            't' => time(),
        ]);

        $plan = Plan::query()->create([
            'group_id' => 1,
            'transfer_enable' => 1024 * 1024,
            'name' => '旗舰套餐',
            'show' => true,
            'renew' => true,
            'sell' => true,
            'sort' => 0,
            'scope' => Plan::SCOPE_NODE,
            'owner_user_id' => $provider->id,
            'node_ids' => [],
            'prices' => [
                Plan::PERIOD_MONTHLY => 1999,
            ],
        ]);

        UserPlanSubscription::query()->create([
            'user_id' => $consumer->id,
            'plan_id' => $plan->id,
            'order_id' => 1001,
            'period' => Plan::PERIOD_MONTHLY,
            'traffic_allowance_kb' => 1024 * 1024,
            'used_traffic_kb' => 128 * 1024,
            'started_at' => time() - 3600,
            'expired_at' => time() + 86400 * 30,
            'status' => UserPlanSubscription::STATUS_ACTIVE,
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $provider->id,
            'name' => 'Tokyo Edge',
            'protocol' => ServerNode::PROTOCOL_VLESS,
            'location_code' => 'JP',
            'location_name' => 'Japan 东京',
            'traffic_limit' => 1024 * 1024,
            'traffic_used' => 512 * 1024,
            'tcping_enabled' => true,
            'tcping_last_status' => 'online',
            'tcping_last_latency_ms' => 42,
            'tcping_last_sampled_at' => time(),
        ]);

        Cache::put(CacheKey::get('SERVER_SERVER_NODE_LAST_LOAD_AT', $node->id), time(), 300);

        NodeTrafficRecord::query()->create([
            'user_id' => $consumer->id,
            'node_id' => $node->id,
            'upload_traffic' => 120000,
            'download_traffic' => 350000,
            'record_date' => now()->toDateString(),
        ]);
        NodeTrafficRecord::query()->create([
            'user_id' => $consumer->id,
            'node_id' => $node->id,
            'upload_traffic' => 80000,
            'download_traffic' => 220000,
            'record_date' => now()->subDay()->toDateString(),
        ]);

        UserOnlineSession::query()->create([
            'user_id' => $consumer->id,
            'node_id' => $node->id,
            'ip_address' => '203.0.113.8',
            'connection_count' => 3,
            'upload_traffic' => 1024,
            'download_traffic' => 4096,
            'last_activity' => now(),
        ]);

        $agent = TcpingAgent::query()->create([
            'user_id' => $provider->id,
            'name' => 'Tokyo Agent',
            'token' => Str::random(48),
            'is_enabled' => true,
            'last_heartbeat_at' => time(),
            'last_sync_at' => time(),
        ]);

        TcpingSample::query()->create([
            'node_id' => $node->id,
            'agent_id' => $agent->id,
            'is_reachable' => true,
            'latency_ms' => 40,
            'is_timeout' => false,
            'sampled_at' => time() - 120,
        ]);
        TcpingSample::query()->create([
            'node_id' => $node->id,
            'agent_id' => $agent->id,
            'is_reachable' => true,
            'latency_ms' => 42,
            'is_timeout' => false,
            'sampled_at' => time() - 60,
        ]);

        TcpingAlert::query()->create([
            'node_id' => $node->id,
            'user_id' => $provider->id,
            'status' => TcpingAlert::STATUS_ACTIVE,
            'started_at' => time() - 600,
            'triggered_at' => time() - 600,
            'latest_error' => 'timeout',
        ]);

        Ticket::query()->create([
            'user_id' => $consumer->id,
            'node_id' => $node->id,
            'assigned_admin_user_id' => $superAdmin->id,
            'subject' => 'Tokyo node unstable',
            'level' => 1,
            'status' => Ticket::STATUS_OPENING,
            'reply_status' => 0,
        ]);

        $order = Order::query()->create([
            'user_id' => $consumer->id,
            'plan_id' => $plan->id,
            'payment_id' => null,
            'period' => Plan::PERIOD_MONTHLY,
            'trade_no' => Str::uuid()->toString(),
            'total_amount' => 1999,
            'type' => Order::TYPE_NEW_PURCHASE,
            'status' => Order::STATUS_COMPLETED,
            'paid_at' => time() - 1800,
        ]);

        OrderRefundRequest::unguarded(function () use ($order, $consumer, $plan, $superAdmin) {
            OrderRefundRequest::query()->create([
                'order_id' => $order->id,
                'trade_no' => 'refund-' . $order->trade_no,
                'user_id' => $consumer->id,
                'plan_id' => $plan->id,
                'assigned_admin_user_id' => $superAdmin->id,
                'status' => OrderRefundRequest::STATUS_PENDING,
                'gateway_amount' => 1999,
                'refund_amount' => 1500,
            ]);
        });

        AuditLog::query()->create([
            'user_id' => $consumer->id,
            'node_id' => $node->id,
            'ip_address' => '203.0.113.8',
            'target_domain' => 'example.org',
            'target_protocol' => 'tls',
            'action_taken' => AuditLog::ACTION_LOGGED,
        ]);

        Sanctum::actingAs($superAdmin);

        $response = $this->getJson('/api/v1/admin/command-center');

        $response->assertOk()
            ->assertJsonStructure([
                'status',
                'message',
                'data' => [
                    'generated_at',
                    'refresh_interval_seconds',
                    'overview' => [
                        'total_users',
                        'live_users',
                        'active_subscriptions',
                        'total_nodes',
                        'online_nodes',
                        'tcping_alerts_active',
                        'revenue_24h_amount',
                    ],
                    'system' => [
                        'schedule_ok',
                        'schedule_last_runtime',
                        'horizon',
                        'logs',
                    ],
                    'traffic_trend',
                    'protocol_distribution',
                    'region_distribution',
                    'top_users',
                    'node_watchlist',
                    'tickets',
                    'refunds',
                    'tcping_agents',
                    'tcping_alerts',
                    'audit_stream',
                ],
            ])
            ->assertJsonPath('data.overview.total_users', 3)
            ->assertJsonPath('data.overview.total_nodes', 1)
            ->assertJsonPath('data.top_users.0.email', 'consumer@example.com')
            ->assertJsonPath('data.node_watchlist.0.owner_email', 'provider@example.com')
            ->assertJsonPath('data.protocol_distribution.0.protocol', 'vless')
            ->assertJsonPath('data.region_distribution.0.location_code', 'JP')
            ->assertJsonPath('data.tcping_agents.0.owner_email', 'provider@example.com')
            ->assertJsonPath('data.tickets.0.user_email', 'consumer@example.com')
            ->assertJsonPath('data.refunds.0.user_email', 'consumer@example.com')
            ->assertJsonPath('data.audit_stream.0.user_email', 'consumer@example.com');
    }

    public function test_non_super_admin_cannot_fetch_command_center_snapshot(): void
    {
        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => false,
        ]);

        Sanctum::actingAs($admin);

        $this->getJson('/api/v1/admin/command-center')
            ->assertStatus(403);
    }
}
