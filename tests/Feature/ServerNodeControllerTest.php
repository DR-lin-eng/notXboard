<?php

namespace Tests\Feature;

use App\Models\User;
use App\Models\ServerNode;
use App\Services\AccessControlService;
use App\Services\LimitControlService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Foundation\Testing\WithFaker;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class ServerNodeControllerTest extends TestCase
{
    use RefreshDatabase, WithFaker;

    protected User $user;

    protected function setUp(): void
    {
        parent::setUp();
        
        $this->user = User::factory()->create([
            'trust_level' => 2,
            'is_super_admin' => false,
        ]);
    }

    public function test_user_can_list_their_server_nodes()
    {
        Sanctum::actingAs($this->user);

        // Create some server nodes for the user
        ServerNode::factory()->count(3)->create(['user_id' => $this->user->id]);
        
        // Create a node for another user (should not be included)
        $otherUser = User::factory()->create();
        ServerNode::factory()->create(['user_id' => $otherUser->id]);

        $response = $this->getJson('/api/v1/user/server-nodes');

        $response->assertStatus(200)
                ->assertJsonStructure([
                    'data' => [
                        '*' => [
                            'id',
                            'name',
                            'host',
                            'port',
                            'protocol',
                            'status',
                            'traffic_limit',
                            'traffic_used',
                            'is_active',
                            'created_at',
                            'updated_at',
                        ]
                    ]
                ]);

        $this->assertCount(3, $response->json('data'));
    }

    public function test_user_can_create_server_node()
    {
        Sanctum::actingAs($this->user);

        $nodeData = [
            'name' => 'Test Node',
            'host' => '192.168.1.100',
            'port' => 8080,
            'protocol' => 'vmess',
            'location_code' => 'HK',
            'location_name' => 'Hong Kong',
            'traffic_limit' => 1000000, // 1GB in KB
            'device_limit' => 5,
            'connection_limit' => 10,
            'speed_limit_up' => 100,
            'speed_limit_down' => 100,
        ];

        $response = $this->postJson('/api/v1/user/server-nodes', $nodeData);

        $response->assertStatus(201)
                ->assertJsonStructure([
                    'message',
                    'data' => [
                        'id',
                        'name',
                        'host',
                        'port',
                        'protocol',
                        'status',
                        'user_id',
                    ]
                ]);

        $this->assertDatabaseHas('server_nodes', [
            'name' => 'Test Node',
            'host' => '192.168.1.100',
            'port' => 8080,
            'protocol' => 'vmess',
            'location_code' => 'HK',
            'location_name' => 'Hong Kong',
            'user_id' => $this->user->id,
            'status' => ServerNode::STATUS_INACTIVE,
        ]);
    }

    public function test_user_cannot_create_server_node_without_location()
    {
        Sanctum::actingAs($this->user);

        $response = $this->postJson('/api/v1/user/server-nodes', [
            'name' => 'Node without location',
            'host' => 'example.com',
            'port' => 443,
            'protocol' => 'vless',
        ]);

        $response->assertStatus(422)
            ->assertJsonValidationErrors(['location_code', 'location_name']);
    }

    public function test_user_can_view_their_server_node()
    {
        Sanctum::actingAs($this->user);

        $node = ServerNode::factory()->create(['user_id' => $this->user->id]);

        $response = $this->getJson("/api/v1/user/server-nodes/{$node->id}");

        $response->assertStatus(200)
                ->assertJsonStructure([
                    'data' => [
                        'id',
                        'name',
                        'host',
                        'port',
                        'protocol',
                        'settings',
                        'status',
                        'traffic_limit',
                        'traffic_used',
                        'access_control',
                        'is_active',
                        'created_at',
                        'updated_at',
                    ]
                ]);

        $this->assertEquals($node->id, $response->json('data.id'));
    }

    public function test_user_cannot_view_other_users_server_node()
    {
        Sanctum::actingAs($this->user);

        $otherUser = User::factory()->create();
        $node = ServerNode::factory()->create(['user_id' => $otherUser->id]);

        $response = $this->getJson("/api/v1/user/server-nodes/{$node->id}");

        $response->assertStatus(404);
    }

    public function test_user_can_update_their_server_node()
    {
        Sanctum::actingAs($this->user);

        $node = ServerNode::factory()->create(['user_id' => $this->user->id]);

        $updateData = [
            'name' => 'Updated Node Name',
            'traffic_limit' => 2000000, // 2GB in KB
        ];

        $response = $this->putJson("/api/v1/user/server-nodes/{$node->id}", $updateData);

        $response->assertStatus(200)
                ->assertJsonStructure([
                    'message',
                    'data'
                ]);

        $this->assertDatabaseHas('server_nodes', [
            'id' => $node->id,
            'name' => 'Updated Node Name',
            'traffic_limit' => 2000000,
        ]);
    }

    public function test_user_can_delete_their_server_node()
    {
        Sanctum::actingAs($this->user);

        $node = ServerNode::factory()->create(['user_id' => $this->user->id]);

        $response = $this->deleteJson("/api/v1/user/server-nodes/{$node->id}");

        $response->assertStatus(200)
                ->assertJson(['message' => 'Server node deleted successfully']);

        $this->assertDatabaseMissing('server_nodes', ['id' => $node->id]);
    }

    public function test_user_can_deploy_their_server_node()
    {
        Sanctum::actingAs($this->user);

        $node = ServerNode::factory()->create([
            'user_id' => $this->user->id,
            'status' => ServerNode::STATUS_INACTIVE,
        ]);

        $response = $this->postJson("/api/v1/user/server-nodes/{$node->id}/deploy");

        $response->assertStatus(200)
                ->assertJsonStructure([
                    'message',
                    'data'
                ]);

        $this->assertDatabaseHas('server_nodes', [
            'id' => $node->id,
            'status' => ServerNode::STATUS_ACTIVE,
        ]);
    }

    public function test_user_can_get_server_node_status()
    {
        Sanctum::actingAs($this->user);

        $node = ServerNode::factory()->create(['user_id' => $this->user->id]);

        $response = $this->getJson("/api/v1/user/server-nodes/{$node->id}/status");

        $response->assertStatus(200)
                ->assertJsonStructure([
                    'data' => [
                        'id',
                        'name',
                        'status',
                        'is_active',
                        'online_users',
                        'traffic_usage_percentage',
                        'is_traffic_exceeded',
                        'last_check_at',
                    ]
                ]);
    }

    public function test_user_can_configure_access_control()
    {
        Sanctum::actingAs($this->user);

        $node = ServerNode::factory()->create(['user_id' => $this->user->id]);
        $targetUser = User::factory()->create();

        $accessControl = [
            'min_trust_level' => 2,
            'authorized_users' => [$targetUser->id],
        ];

        $response = $this->postJson("/api/v1/user/server-nodes/{$node->id}/access", $accessControl);

        $response->assertStatus(200)
                ->assertJsonStructure([
                    'message',
                    'data'
                ]);

        $this->assertDatabaseHas('server_nodes', [
            'id' => $node->id,
            'access_control' => json_encode(['min_trust_level' => 2, 'authorized_users' => [$targetUser->id]]),
        ]);
    }

    public function test_validation_errors_for_invalid_server_node_data()
    {
        Sanctum::actingAs($this->user);

        $invalidData = [
            'name' => '', // Required field
            'host' => '', // Required field
            'port' => 70000, // Invalid port
            'protocol' => 'invalid_protocol', // Invalid protocol
        ];

        $response = $this->postJson('/api/v1/user/server-nodes', $invalidData);

        $response->assertStatus(422)
                ->assertJsonStructure([
                    'message',
                    'errors' => [
                        'name',
                        'host',
                        'port',
                        'protocol',
                    ]
                ]);
    }

    public function test_non_super_admin_cannot_set_concurrent_ip_limit()
    {
        Sanctum::actingAs($this->user);

        $nodeData = [
            'name' => 'Test Node',
            'host' => '192.168.1.100',
            'port' => 8080,
            'protocol' => 'vmess',
            'location_code' => 'HK',
            'location_name' => 'Hong Kong',
            'concurrent_ip_limit' => 5, // Should be ignored for non-super-admin
        ];

        $response = $this->postJson('/api/v1/user/server-nodes', $nodeData);

        $response->assertStatus(201);

        $this->assertDatabaseHas('server_nodes', [
            'name' => 'Test Node',
            'concurrent_ip_limit' => 0, // Should be 0 (default) since user is not super admin
        ]);
    }

    public function test_super_admin_can_set_concurrent_ip_limit()
    {
        $superAdmin = User::factory()->create(['is_super_admin' => true]);
        Sanctum::actingAs($superAdmin);

        $nodeData = [
            'name' => 'Test Node',
            'host' => '192.168.1.100',
            'port' => 8080,
            'protocol' => 'vmess',
            'location_code' => 'HK',
            'location_name' => 'Hong Kong',
            'concurrent_ip_limit' => 5,
        ];

        $response = $this->postJson('/api/v1/user/server-nodes', $nodeData);

        $response->assertStatus(201);

        $this->assertDatabaseHas('server_nodes', [
            'name' => 'Test Node',
            'concurrent_ip_limit' => 5,
        ]);
    }
}
