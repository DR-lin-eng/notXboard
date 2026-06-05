<?php
namespace App\Logging;

use Monolog\Logger;

class MysqlLogger
{
    public function __invoke(array $config)
    {
        $level = Logger::toMonologLevel($config['level'] ?? Logger::WARNING);
        $bubble = (bool) ($config['bubble'] ?? false);

        return tap(new Logger('mysql'), function (Logger $logger) use ($level, $bubble) {
            $logger->pushHandler(new MysqlLoggerHandler($level, $bubble));
        });
    }
}
