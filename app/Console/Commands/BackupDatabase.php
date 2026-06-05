<?php

namespace App\Console\Commands;

use App\Services\TelegramService;
use Illuminate\Console\Command;
use Google\Cloud\Storage\StorageClient;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Log;
use Symfony\Component\Process\Process;
use Throwable;

class BackupDatabase extends Command
{
    protected $signature = 'backup:database {upload?}';
    protected $description = '备份数据库并上传到 Google Cloud Storage';

    public function handle(): int
    {
        $isUpload = filter_var((string) $this->argument('upload'), FILTER_VALIDATE_BOOLEAN);
        $databaseBackupPath = null;
        $compressedBackupPath = null;

        if ($isUpload && !$this->validateUploadConfig()) {
            return self::FAILURE;
        }

        try {
            File::ensureDirectoryExists(storage_path('backup'));

            if (config('database.default') === 'mysql') {
                $databaseBackupPath = storage_path('backup/' . now()->format('Y-m-d_H-i-s') . '_' . config('database.connections.mysql.database') . '_database_backup.sql');
                $this->info('1️⃣：开始备份Mysql');
                \Spatie\DbDumper\Databases\MySql::create()
                    ->setHost(config('database.connections.mysql.host'))
                    ->setPort(config('database.connections.mysql.port'))
                    ->setDbName(config('database.connections.mysql.database'))
                    ->setUserName(config('database.connections.mysql.username'))
                    ->setPassword(config('database.connections.mysql.password'))
                    ->dumpToFile($databaseBackupPath);
                $this->info('2️⃣：Mysql备份完成');
            } elseif (config('database.default') === 'sqlite') {
                $databaseBackupPath = storage_path('backup/' . now()->format('Y-m-d_H-i-s') . '_sqlite_database_backup.sql');
                $this->info('1️⃣：开始备份Sqlite');
                \Spatie\DbDumper\Databases\Sqlite::create()
                    ->setDbName(config('database.connections.sqlite.database'))
                    ->dumpToFile($databaseBackupPath);
                $this->info('2️⃣：Sqlite备份完成');
            } else {
                $this->error('备份失败，你的数据库不是 sqlite 或 mysql');
                return self::FAILURE;
            }

            $this->info('3️⃣：开始压缩备份文件');
            $compressedBackupPath = $databaseBackupPath . '.gz';
            $gzipCommand = new Process(['gzip', '-c', $databaseBackupPath]);
            $gzipCommand->setTimeout(120);
            $gzipCommand->run();

            if (!$gzipCommand->isSuccessful()) {
                $this->error('😔：文件压缩失败');
                $this->safeBackupLog('error', "😔：文件压缩失败\n" . $gzipCommand->getErrorOutput());
                $this->safeDeleteBackupFile($databaseBackupPath);
                return self::FAILURE;
            }

            file_put_contents($compressedBackupPath, $gzipCommand->getOutput());
            $this->safeDeleteBackupFile($databaseBackupPath);
            $this->info('4️⃣：文件压缩成功');

            if (!$isUpload) {
                $this->info("🎉：数据库成功备份到：{$compressedBackupPath}");
                return self::SUCCESS;
            }

            $this->info('5️⃣：开始将备份上传到Google Cloud');
            $storage = new StorageClient([
                'keyFilePath' => config('services.google_cloud.key_file'),
            ]);
            $bucket = $storage->bucket(config('services.google_cloud.storage_bucket'));
            $objectName = 'backup/' . now()->format('Y-m-d_H-i-s') . '_database_backup.sql.gz';
            $bucket->upload(fopen($compressedBackupPath, 'r'), [
                'name' => $objectName,
            ]);

            $this->safeBackupLog('info', "🎉：数据库备份已上传到 Google Cloud Storage: {$objectName}");
            $this->info("🎉：数据库备份已上传到 Google Cloud Storage: {$objectName}");
            $this->safeDeleteBackupFile($compressedBackupPath);

            return self::SUCCESS;
        } catch (Throwable $exception) {
            $this->safeBackupLog('error', "😔：数据库备份失败\n" . $exception);
            $this->error("😔：数据库备份失败\n" . $exception->getMessage());
            $this->notifyBackupFailure($exception);

            $this->safeDeleteBackupFile($compressedBackupPath);
            $this->safeDeleteBackupFile($databaseBackupPath);

            return self::FAILURE;
        }
    }

    private function validateUploadConfig(): bool
    {
        $requiredConfigs = [
            'services.google_cloud.key_file',
            'services.google_cloud.storage_bucket',
        ];

        if (config('database.default') === 'mysql') {
            $requiredConfigs = array_merge($requiredConfigs, [
                'database.connections.mysql.host',
                'database.connections.mysql.database',
                'database.connections.mysql.username',
            ]);
        } elseif (config('database.default') === 'sqlite') {
            $requiredConfigs[] = 'database.connections.sqlite.database';
        }

        foreach ($requiredConfigs as $configKey) {
            if (blank(config($configKey))) {
                $this->error("❌：缺少必要配置项: {$configKey}，取消备份");
                return false;
            }
        }

        return true;
    }

    private function notifyBackupFailure(Throwable $exception): void
    {
        try {
            app(TelegramService::class)->sendOpsAlert(
                "*[Ops Alert]*\nsource: backup\nevent: backup:database:failed\ntime: " . now()->toDateTimeString() .
                "\n\nmessage: " . $exception->getMessage(),
                [
                    'source' => 'backup',
                    'event' => 'backup:database:failed',
                ]
            );
        } catch (Throwable $telegramException) {
            $this->safeBackupLog('error', 'Telegram 告警发送失败: ' . $telegramException->getMessage());
        }
    }

    private function safeDeleteBackupFile(?string $path): void
    {
        if (!is_string($path) || $path === '') {
            return;
        }

        try {
            if (File::exists($path)) {
                File::delete($path);
            }
        } catch (Throwable $exception) {
            $this->safeBackupLog('warning', '清理备份文件失败', [
                'path' => $path,
                'error' => $exception->getMessage(),
            ]);
        }
    }

    private function safeBackupLog(string $level, string $message, array $context = []): void
    {
        try {
            Log::channel('backup')->log($level, $message, $context);
        } catch (Throwable $exception) {
            error_log(sprintf(
                '[backup-log:%s] %s | fallback=%s | context=%s',
                $level,
                $message,
                $exception->getMessage(),
                json_encode($context, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES)
            ));
        }
    }
}
