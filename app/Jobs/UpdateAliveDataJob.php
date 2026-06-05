<?php

namespace App\Jobs;

use App\Services\LegacyTrafficDispatchService;
use Illuminate\Bus\Queueable;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Bus\Dispatchable;
use Illuminate\Queue\InteractsWithQueue;
use Illuminate\Queue\SerializesModels;
use Illuminate\Support\Facades\Log;

class UpdateAliveDataJob implements ShouldQueue
{
  use Dispatchable, InteractsWithQueue, Queueable, SerializesModels;

  public function __construct(
    private readonly array $data,
    private readonly string $nodeType,
    private readonly int $nodeId
  ) {
    $this->onQueue('online_sync');
  }

  public function handle(): void
  {
    try {
      app(LegacyTrafficDispatchService::class)->updateAliveData(
        $this->data,
        $this->nodeType,
        $this->nodeId
      );
    } catch (\Throwable $e) {
      Log::error('UpdateAliveDataJob failed', [
        'error' => $e->getMessage(),
      ]);
      $this->fail($e);
    }
  }


}
