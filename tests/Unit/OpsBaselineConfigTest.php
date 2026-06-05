<?php

namespace Tests\Unit;

use Tests\TestCase;

class OpsBaselineConfigTest extends TestCase
{
    public function test_horizon_has_production_supervisors(): void
    {
        $config = require config_path('horizon.php');

        $this->assertArrayHasKey('production', $config['environments']);
        $this->assertArrayHasKey('Xboard-core', $config['environments']['production']);
        $this->assertArrayHasKey('Xboard-notify', $config['environments']['production']);
    }

    public function test_logging_default_is_not_forced_to_mysql(): void
    {
        $config = require config_path('logging.php');

        $this->assertNotSame('mysql', $config['default']);
        $this->assertArrayHasKey('stack', $config['channels']);
        $this->assertContains('stderr', $config['channels']['stack']['channels']);
    }

    public function test_queue_retry_after_exceeds_horizon_worker_timeouts(): void
    {
        $queue = require config_path('queue.php');
        $horizon = require config_path('horizon.php');

        $retryAfter = $queue['connections']['redis']['retry_after'];
        $coreTimeout = $horizon['environments']['production']['Xboard-core']['timeout'];
        $notifyTimeout = $horizon['environments']['production']['Xboard-notify']['timeout'];

        $this->assertGreaterThan($coreTimeout, $retryAfter);
        $this->assertGreaterThan($notifyTimeout, $retryAfter);
    }

    public function test_queue_block_for_is_configured_for_redis_workers(): void
    {
        $queue = require config_path('queue.php');

        $this->assertIsInt($queue['connections']['redis']['block_for']);
        $this->assertGreaterThan(0, $queue['connections']['redis']['block_for']);
    }
}
