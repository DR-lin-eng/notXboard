<?php

namespace App\Console\Commands;

use Illuminate\Console\Command;
use Symfony\Component\Process\Process;

class Test extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'test
        {--filter= : Only run tests matching the given pattern}
        {--testsuite= : Only run the given testsuite}
        {--stop-on-failure : Stop after the first failure}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Run the project PHPUnit test suite when dev dependencies are installed';

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
    public function handle()
    {
        $phpunitBinary = $this->resolvePhpUnitBinary();
        if (!$phpunitBinary) {
            $this->error('PHPUnit is not installed in the current environment. Run composer install with dev dependencies first.');
            return self::FAILURE;
        }

        $command = array_merge(
            [PHP_BINARY, $phpunitBinary],
            $this->buildPhpUnitArguments()
        );

        $process = new Process($command, base_path());
        $process->setTimeout(null);
        $process->setTty(false);
        $process->run(function (string $type, string $buffer): void {
            $this->output->write($buffer);
        });

        return $process->isSuccessful() ? self::SUCCESS : ($process->getExitCode() ?? self::FAILURE);
    }

    private function resolvePhpUnitBinary(): ?string
    {
        $candidates = [
            base_path('vendor/bin/phpunit'),
            base_path('vendor/phpunit/phpunit/phpunit'),
        ];

        foreach ($candidates as $candidate) {
            if (is_file($candidate)) {
                return $candidate;
            }
        }

        return null;
    }

    private function buildPhpUnitArguments(): array
    {
        $arguments = [];

        if ($filter = $this->option('filter')) {
            $arguments[] = '--filter';
            $arguments[] = (string) $filter;
        }

        if ($testsuite = $this->option('testsuite')) {
            $arguments[] = '--testsuite';
            $arguments[] = (string) $testsuite;
        }

        if ((bool) $this->option('stop-on-failure')) {
            $arguments[] = '--stop-on-failure';
        }

        return $arguments;
    }
}
