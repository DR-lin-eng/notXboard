<?php

namespace Tests\Feature;

use App\Models\ServerNode;
use App\Models\User;
use App\Services\TcpingService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class TcpingAgentApiTest extends TestCase
{
    use RefreshDatabase;

    public function test_agent_can_fetch_config_send_heartbeat_and_upload_samples(): void
    {
        $owner = User::factory()->create();
        $node = ServerNode::factory()->create([
            'user_id' => $owner->id,
            'tcping_enabled' => true,
            'tcping_host' => 'probe.example.com',
            'tcping_port' => 443,
            'tcping_interval_seconds' => 60,
            'tcping_timeout_ms' => 3000,
            'tcping_alert_after_seconds' => 300,
            'tcping_recover_after_seconds' => 120,
        ]);

        $agent = app(TcpingService::class)->createAgentForUser($owner, 'edge-prober');

        $configResponse = $this->getJson('/api/v1/tcping/agent/config?token=' . $agent->token);
        $configResponse->assertOk()
            ->assertJsonPath('data.targets.0.node_id', $node->id)
            ->assertJsonPath('data.targets.0.host', 'probe.example.com');

        $heartbeatResponse = $this->postJson('/api/v1/tcping/agent/heartbeat?token=' . $agent->token);
        $heartbeatResponse->assertOk();
        $this->assertNotNull($agent->fresh()->last_heartbeat_at);

        $sampledAt = time();
        $samplesResponse = $this->postJson('/api/v1/tcping/agent/samples?token=' . $agent->token, [
            'samples' => [
                [
                    'node_id' => $node->id,
                    'is_reachable' => true,
                    'latency_ms' => 88,
                    'is_timeout' => false,
                    'sampled_at' => $sampledAt,
                ],
            ],
        ]);
        $samplesResponse->assertOk()
            ->assertJsonPath('data.accepted', 1);

        $this->assertDatabaseHas('tcping_samples', [
            'node_id' => $node->id,
            'agent_id' => $agent->id,
            'is_reachable' => 1,
            'latency_ms' => 88,
        ]);

        $node->refresh();
        $this->assertSame('online', $node->tcping_last_status);
        $this->assertSame(88, (int) $node->tcping_last_latency_ms);
        $this->assertSame($sampledAt, (int) $node->tcping_last_sampled_at);
    }
}
