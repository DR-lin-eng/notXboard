<?php

namespace Tests\Feature;

use App\Models\AuditLog;
use App\Models\Server;
use App\Models\ServerNode;
use App\Models\User;
use App\Models\UserOnlineSession;
use App\Utils\CacheKey;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Cache;
use Tests\TestCase;

class UniProxyUserEndpointTest extends TestCase
{
    use RefreshDatabase;

    private function endpoint(ServerNode $node, string $path, ?string $token = null): string
    {
        return sprintf(
            '/api/v1/server/UniProxy/%s?node_id=%d&token=%s',
            ltrim($path, '/'),
            $node->v2bx_node_id,
            $token ?? $node->v2bx_token
        );
    }

    public function test_uniproxy_user_endpoint_returns_users_for_server_node(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
            'subscription_credential_version' => 1,
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'test-node-token',
            'v2bx_node_id' => 1001,
        ]);

        $response = $this->getJson($this->endpoint($node, 'user'));

        $response->assertOk()
            ->assertJsonStructure([
                'users' => [
                    '*' => [
                        'id',
                        'uuid',
                        'speed_limit',
                        'device_limit',
                        'connection_limit',
                        'trust_level',
                        'is_silenced',
                    ],
                ],
            ])
            ->assertJsonPath('users.0.id', $owner->id);

        $this->assertNotEmpty($response->json('users.0.uuid'));
    }

    public function test_uniproxy_alivelist_endpoint_supports_server_nodes(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'test-node-token-alive',
            'v2bx_node_id' => 1002,
            'device_limit' => 2,
        ]);

        $response = $this->getJson($this->endpoint($node, 'alivelist'));

        $response->assertOk()
            ->assertJsonStructure([
                'alive',
            ]);
    }

    public function test_uniproxy_rejects_owner_api_key_as_node_token(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'abbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
            'api_key' => 'owner-api-key-token',
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'strict-node-token',
            'v2bx_node_id' => 1006,
        ]);

        $response = $this->getJson($this->endpoint($node, 'user', 'owner-api-key-token'));

        $response->assertStatus(400)
            ->assertJsonPath('message', 'Invalid server token');
    }

    public function test_uniproxy_user_endpoint_supports_legacy_v2_server_authentication(): void
    {
        config(['v2board.server_token' => 'legacy-server-token']);

        $user = User::factory()->create([
            'group_id' => 1,
            'transfer_enable' => 1024,
            'expired_at' => time() + 3600,
            'uuid' => '11111111-1111-4111-8111-111111111111',
        ]);

        $server = Server::query()->create([
            'type' => Server::TYPE_VMESS,
            'code' => '2001',
            'parent_id' => null,
            'group_ids' => [1],
            'route_ids' => [],
            'name' => 'Legacy UniProxy Node',
            'rate' => 1,
            'tags' => [],
            'host' => 'legacy.example.com',
            'port' => '443',
            'server_port' => 443,
            'protocol_settings' => [],
            'show' => true,
            'sort' => 1,
        ]);

        $response = $this->getJson(
            sprintf(
                '/api/v1/server/UniProxy/user?node_id=%s&token=%s',
                $server->code,
                'legacy-server-token'
            )
        );

        $response->assertOk()
            ->assertJsonPath('users.0.id', $user->id);
    }

    public function test_uniproxy_alive_endpoint_records_online_sessions_for_server_nodes(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'cccccccc-cccc-4ccc-8ccc-cccccccccccc',
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'test-node-token-session',
            'v2bx_node_id' => 1003,
        ]);

        $response = $this->postJson($this->endpoint($node, 'alive'), [
            (string) $owner->id => ['1.1.1.1', '2001:db8::1'],
        ]);

        $response->assertOk()
            ->assertJsonPath('data', true);

        $this->assertSame(2, UserOnlineSession::query()->where('node_id', $node->id)->count());
        $this->assertDatabaseHas('user_online_sessions', [
            'user_id' => $owner->id,
            'node_id' => $node->id,
            'ip_address' => '1.1.1.1',
        ]);
    }

    public function test_uniproxy_alive_endpoint_drops_unauthorized_users_for_server_nodes(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'dccccccc-cccc-4ccc-8ccc-cccccccccccc',
        ]);
        $unauthorized = User::factory()->create([
            'uuid' => 'eccccccc-cccc-4ccc-8ccc-cccccccccccc',
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'test-node-token-session-filter',
            'v2bx_node_id' => 1007,
            'access_control' => [
                'min_trust_level' => 4,
            ],
        ]);

        $response = $this->postJson($this->endpoint($node, 'alive'), [
            (string) $owner->id => ['1.1.1.1'],
            (string) $unauthorized->id => ['2.2.2.2'],
        ]);

        $response->assertOk()
            ->assertJsonPath('data', true);

        $this->assertDatabaseHas('user_online_sessions', [
            'user_id' => $owner->id,
            'node_id' => $node->id,
            'ip_address' => '1.1.1.1',
        ]);
        $this->assertDatabaseMissing('user_online_sessions', [
            'user_id' => $unauthorized->id,
            'node_id' => $node->id,
            'ip_address' => '2.2.2.2',
        ]);
    }

    public function test_uniproxy_push_endpoint_drops_unauthorized_users_for_server_nodes(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'fccccccc-cccc-4ccc-8ccc-cccccccccccc',
        ]);
        $unauthorized = User::factory()->create([
            'uuid' => '0ccccccc-cccc-4ccc-8ccc-cccccccccccc',
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'test-node-token-push-filter',
            'v2bx_node_id' => 1008,
            'traffic_multiplier' => 1,
            'access_control' => [
                'min_trust_level' => 4,
            ],
        ]);

        $response = $this->postJson($this->endpoint($node, 'push'), [
            (string) $owner->id => [120, 80],
            (string) $unauthorized->id => [999, 999],
        ]);

        $response->assertOk()
            ->assertJsonPath('data', true);

        $owner->refresh();
        $unauthorized->refresh();

        $this->assertSame(120, (int) $owner->u);
        $this->assertSame(80, (int) $owner->d);
        $this->assertSame(0, (int) $unauthorized->u);
        $this->assertSame(0, (int) $unauthorized->d);

        $this->assertDatabaseHas('node_traffic_records', [
            'user_id' => $owner->id,
            'node_id' => $node->id,
        ]);
        $this->assertDatabaseMissing('node_traffic_records', [
            'user_id' => $unauthorized->id,
            'node_id' => $node->id,
        ]);
    }

    public function test_uniproxy_status_endpoint_updates_server_node_load_cache(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'dddddddd-dddd-4ddd-8ddd-dddddddddddd',
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'test-node-token-status',
            'v2bx_node_id' => 1004,
        ]);

        $response = $this->postJson($this->endpoint($node, 'status'), [
            'cpu' => 12.5,
            'mem' => ['total' => 1024, 'used' => 256],
            'swap' => ['total' => 512, 'used' => 0],
            'disk' => ['total' => 2048, 'used' => 512],
        ]);

        $response->assertOk()
            ->assertJsonPath('data', true)
            ->assertJsonPath('code', 0);

        $payload = Cache::get(CacheKey::get('SERVER_SERVER_NODE_LOAD_STATUS', $node->id));
        $this->assertIsArray($payload);
        $this->assertSame(12.5, $payload['cpu']);
        $this->assertSame(1024, $payload['mem']['total']);
    }

    public function test_uniproxy_audit_endpoint_records_audit_logs_for_server_nodes(): void
    {
        $owner = User::factory()->create([
            'uuid' => 'eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee',
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'v2bx_token' => 'test-node-token-audit',
            'v2bx_node_id' => 1005,
        ]);

        $response = $this->postJson($this->endpoint($node, 'audit'), [
            'user_id' => $owner->id,
            'ip_address' => '1.1.1.1',
            'target_domain' => 'example.com',
            'target_protocol' => 'tls',
            'action_taken' => AuditLog::ACTION_LOGGED,
        ]);

        $response->assertOk()
            ->assertJsonPath('success', true)
            ->assertJsonPath('data.logged', true);

        $this->assertDatabaseHas('audit_logs', [
            'user_id' => $owner->id,
            'node_id' => $node->id,
            'ip_address' => '1.1.1.1',
            'target_domain' => 'example.com',
            'target_protocol' => 'tls',
            'action_taken' => AuditLog::ACTION_LOGGED,
        ]);
    }
}
