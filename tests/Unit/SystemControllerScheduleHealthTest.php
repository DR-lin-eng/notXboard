<?php

namespace Tests\Unit;

use App\Http\Controllers\V2\Admin\SystemController;
use App\Utils\CacheKey;
use Illuminate\Support\Facades\Cache;
use Tests\TestCase;

class SystemControllerScheduleHealthTest extends TestCase
{
    public function test_schedule_status_depends_on_recent_heartbeat(): void
    {
        $controller = new SystemController();
        $method = new \ReflectionMethod($controller, 'getScheduleStatus');
        $method->setAccessible(true);
        $cacheKey = CacheKey::get('SCHEDULE_LAST_CHECK_AT', null);

        Cache::forget($cacheKey);
        $this->assertFalse($method->invoke($controller));

        Cache::put($cacheKey, time() - 400, now()->addMinutes(15));
        $this->assertFalse($method->invoke($controller));

        Cache::put($cacheKey, time() - 20, now()->addMinutes(15));
        $this->assertTrue($method->invoke($controller));
    }
}
