<?php

namespace App\Services;

use Illuminate\Support\Facades\Bus;

class CoreJobDispatchService
{
    public function dispatch(object $job): mixed
    {
        if ($this->shouldDispatchSync()) {
            return Bus::dispatchSync($job);
        }

        return Bus::dispatch($job);
    }

    public function shouldDispatchSync(): bool
    {
        return (bool) config('ops.core_job_sync_execution', true);
    }
}
