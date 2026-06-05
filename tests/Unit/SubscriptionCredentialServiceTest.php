<?php

namespace Tests\Unit;

use App\Models\User;
use App\Services\SubscriptionCredentialService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class SubscriptionCredentialServiceTest extends TestCase
{
    use RefreshDatabase;

    public function test_effective_uuid_changes_only_when_rotation_is_enabled(): void
    {
        $user = User::factory()->create([
            'uuid' => '123e4567-e89b-12d3-a456-426614174000',
            'subscription_credential_version' => 2,
        ]);

        admin_setting(['rotate_subscription_credentials_daily' => 0]);
        $service = app(SubscriptionCredentialService::class);
        $this->assertSame($user->uuid, $service->getEffectiveUuid($user));

        admin_setting(['rotate_subscription_credentials_daily' => 1]);
        $rotated = $service->getEffectiveUuid($user);

        $this->assertNotSame($user->uuid, $rotated);
        $this->assertMatchesRegularExpression('/^[a-f0-9-]{36}$/', $rotated);
    }
}
