<?php

namespace Tests\Unit;

use App\Services\TelegramService;
use Mockery;
use RuntimeException;
use Symfony\Component\Console\Command\Command as SymfonyCommand;
use Tests\TestCase;

class BackupDatabaseCommandTest extends TestCase
{
    public function test_backup_handles_dumper_exception_without_secondary_cleanup_error(): void
    {
        config([
            'database.default' => 'mysql',
            'database.connections.mysql.host' => '127.0.0.1',
            'database.connections.mysql.port' => 3306,
            'database.connections.mysql.database' => 'test_db',
            'database.connections.mysql.username' => 'root',
            'database.connections.mysql.password' => 'secret',
        ]);

        $telegramService = Mockery::mock(TelegramService::class);
        $telegramService->shouldReceive('sendOpsAlert')->once();
        $this->app->instance(TelegramService::class, $telegramService);

        $mysqlDumper = Mockery::mock('alias:Spatie\DbDumper\Databases\MySql');
        $mysqlDumper->shouldReceive('create')
            ->once()
            ->andThrow(new RuntimeException('forced backup failure before compression'));

        $this->artisan('backup:database')
            ->assertExitCode(SymfonyCommand::FAILURE);
    }
}
