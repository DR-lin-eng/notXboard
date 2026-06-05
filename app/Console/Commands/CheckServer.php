<?php

namespace App\Console\Commands;

use App\Models\ServerNode;
use App\Services\ServerService;
use App\Services\TelegramService;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Cache;

class CheckServer extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'check:server';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = '节点检查任务';

    /**
     * Create a new command instance.
     *
     * @return void
     */
    public function __construct()
    {
        parent::__construct();
    }

    /**
     * Execute the console command.
     *
     * @return mixed
     */
    public function handle(): int
    {
        $this->checkLegacyServersOffline();
        $this->checkServerNodesOffline();
        return self::SUCCESS;
    }

    private function checkLegacyServersOffline(): void
    {
        $servers = ServerService::getAllServers();
        foreach ($servers as $server) {
            if ($server['parent_id']) continue;
            if ($server['last_check_at'] && (time() - $server['last_check_at']) > 1800) {
                $this->sendOfflineAlertOnce(
                    'legacy-server:' . (int) $server->id,
                    sprintf(
                        "节点离线\n节点名称：%s\n节点地址：%s\n协议：%s\n时间：%s",
                        (string) $server['name'],
                        (string) $server['host'],
                        (string) $server['type'],
                        now()->toDateTimeString()
                    )
                );
                continue;
            }

            Cache::forget($this->offlineAlertCacheKey('legacy-server:' . (int) $server->id));
        }
    }

    private function checkServerNodesOffline(): void
    {
        ServerNode::query()
            ->where('status', ServerNode::STATUS_ACTIVE)
            ->get()
            ->each(function (ServerNode $node): void {
                $identifier = 'server-node:' . (int) $node->id;

                if ($node->isReportedOnline()) {
                    Cache::forget($this->offlineAlertCacheKey($identifier));
                    return;
                }

                $lastReportAt = $node->getLastReportAt();
                $createdAtTimestamp = $node->created_at?->getTimestamp();
                if ($lastReportAt === null && $createdAtTimestamp !== null && (time() - $createdAtTimestamp) < $node->getOnlineTimeoutSeconds()) {
                    return;
                }

                $this->sendOfflineAlertOnce(
                    $identifier,
                    sprintf(
                        "V2bX 节点离线\n节点名称：%s\n节点地址：%s\n协议：%s\n最后上报：%s\n时间：%s",
                        (string) $node->name,
                        (string) $node->host,
                        (string) $node->protocol,
                        $lastReportAt ? date('Y-m-d H:i:s', $lastReportAt) : 'never',
                        now()->toDateTimeString()
                    )
                );
            });
    }

    private function sendOfflineAlertOnce(string $identifier, string $message): void
    {
        $cacheKey = $this->offlineAlertCacheKey($identifier);
        if (!Cache::add($cacheKey, time(), 3600)) {
            return;
        }

        app(TelegramService::class)->sendOpsAlert($message, [
            'source' => 'server',
            'identifier' => $identifier,
        ]);
    }

    private function offlineAlertCacheKey(string $identifier): string
    {
        return 'ops:server-offline-alert:' . md5($identifier);
    }
}
