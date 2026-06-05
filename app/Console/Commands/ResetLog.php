<?php

namespace App\Console\Commands;

use App\Models\AuditLog;
use App\Models\Log;
use App\Models\NodeTrafficRecord;
use App\Models\StatServer;
use App\Models\StatUser;
use App\Models\TcpingAlert;
use App\Models\TcpingSample;
use App\Models\UserTrafficUsageLog;
use Illuminate\Console\Command;

class ResetLog extends Command
{
    protected $builder;
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'reset:log';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = '清空日志';

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
        StatUser::where('record_at', '<', strtotime('-2 month', time()))->delete();
        StatServer::where('record_at', '<', strtotime('-2 month', time()))->delete();
        Log::where('created_at', '<', strtotime('-1 month', time()))->delete();
        NodeTrafficRecord::cleanupOldRecords((int) admin_setting('node_traffic_records_retention_days', 7));
        UserTrafficUsageLog::cleanupOldRecords((int) admin_setting('user_traffic_usage_logs_retention_days', 7));
        TcpingSample::cleanupOldRecords((int) admin_setting('tcping_samples_retention_days', 7));
        TcpingAlert::cleanupOldRecords((int) admin_setting('tcping_alerts_retention_days', 7));
        AuditLog::cleanupOldLogs((int) admin_setting('audit_logs_retention_days', 7));
    }
}
