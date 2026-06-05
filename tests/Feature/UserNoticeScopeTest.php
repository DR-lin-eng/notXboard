<?php

namespace Tests\Feature;

use App\Models\Notice;
use App\Models\Plan;
use App\Models\User;
use App\Models\UserPlanSubscription;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class UserNoticeScopeTest extends TestCase
{
    use RefreshDatabase;

    public function test_scoped_notice_is_visible_only_to_matching_plan_subscribers(): void
    {
        $owner = User::factory()->create();
        $subscriber = User::factory()->create();
        $outsider = User::factory()->create();

        $plan = Plan::query()->create([
            'group_id' => 1,
            'transfer_enable' => 1024,
            'name' => 'Owner Plan',
            'show' => true,
            'renew' => true,
            'sell' => true,
            'sort' => 0,
            'scope' => Plan::SCOPE_NODE,
            'owner_user_id' => $owner->id,
            'node_ids' => [],
            'prices' => [
                Plan::PERIOD_MONTHLY => 1000,
            ],
        ]);

        UserPlanSubscription::query()->create([
            'user_id' => $subscriber->id,
            'plan_id' => $plan->id,
            'order_id' => 2001,
            'period' => Plan::PERIOD_MONTHLY,
            'traffic_allowance_kb' => 1024,
            'used_traffic_kb' => 0,
            'started_at' => time(),
            'expired_at' => time() + 86400,
            'status' => UserPlanSubscription::STATUS_ACTIVE,
        ]);

        Notice::query()->create([
            'title' => 'Global notice',
            'content' => 'Visible to everyone',
            'tags' => ['global'],
            'show' => true,
            'popup' => false,
            'scope_type' => Notice::SCOPE_GLOBAL,
            'target_plan_ids' => [],
            'author_user_id' => null,
        ]);

        Sanctum::actingAs($owner);
        $createResponse = $this->postJson('/api/v1/user/notice', [
            'title' => 'Scoped notice',
            'content' => 'Visible only to matching subscribers',
            'tags' => ['owner'],
            'target_plan_ids' => [$plan->id],
        ]);
        $createResponse->assertOk();

        Sanctum::actingAs($subscriber);
        $subscriberFetch = $this->getJson('/api/v1/user/notice/fetch');
        $subscriberFetch->assertOk();
        $subscriberTitles = collect($subscriberFetch->json('data'))->pluck('title')->all();
        $this->assertContains('Global notice', $subscriberTitles);
        $this->assertContains('Scoped notice', $subscriberTitles);

        Sanctum::actingAs($outsider);
        $outsiderFetch = $this->getJson('/api/v1/user/notice/fetch');
        $outsiderFetch->assertOk();
        $outsiderTitles = collect($outsiderFetch->json('data'))->pluck('title')->all();
        $this->assertContains('Global notice', $outsiderTitles);
        $this->assertNotContains('Scoped notice', $outsiderTitles);
    }

    public function test_user_cannot_publish_notice_to_other_users_plan(): void
    {
        $owner = User::factory()->create();
        $otherOwner = User::factory()->create();

        $foreignPlan = Plan::query()->create([
            'group_id' => 1,
            'transfer_enable' => 1024,
            'name' => 'Foreign Plan',
            'show' => true,
            'renew' => true,
            'sell' => true,
            'sort' => 0,
            'scope' => Plan::SCOPE_NODE,
            'owner_user_id' => $otherOwner->id,
            'node_ids' => [],
            'prices' => [
                Plan::PERIOD_MONTHLY => 1000,
            ],
        ]);

        Sanctum::actingAs($owner);
        $response = $this->postJson('/api/v1/user/notice', [
            'title' => 'Forbidden scope',
            'content' => 'Should fail',
            'target_plan_ids' => [$foreignPlan->id],
        ]);

        $response->assertStatus(403);
    }
}
