<?php

namespace App\Jobs;

use App\Services\TelegramService;
use Illuminate\Bus\Queueable;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Bus\Dispatchable;
use Illuminate\Queue\InteractsWithQueue;
use Illuminate\Queue\SerializesModels;

class SendTelegramJob implements ShouldQueue
{
    use Dispatchable, InteractsWithQueue, Queueable, SerializesModels;
    protected $telegramId;
    protected $text;
    protected $parseMode;
    protected $options;
    protected $context;

    public $tries = 3;
    public $timeout = 10;

    /**
     * Create a new job instance.
     *
     * @return void
     */
    public function __construct(
        int $telegramId,
        string $text,
        string $parseMode = 'markdown',
        array $options = [],
        ?array $context = null
    )
    {
        $this->onQueue('send_telegram');
        $this->telegramId = $telegramId;
        $this->text = $text;
        $this->parseMode = $parseMode;
        $this->options = $options;
        $this->context = $context;
    }

    /**
     * Execute the job.
     *
     * @return void
     */
    public function handle()
    {
        $telegramService = new TelegramService();
        if (is_array($this->context) && !empty($this->context)) {
            $telegramService->sendContextMessage($this->telegramId, $this->text, $this->context, $this->parseMode, $this->options);
            return;
        }

        $telegramService->sendMessage($this->telegramId, $this->text, $this->parseMode, $this->options);
    }
}
