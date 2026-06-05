<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('server_nodes')) {
            return;
        }

        Schema::table('server_nodes', function (Blueprint $table) {
            if (!Schema::hasColumn('server_nodes', 'location_code')) {
                $table->string('location_code', 16)->nullable()->after('protocol');
            }
            if (!Schema::hasColumn('server_nodes', 'location_name')) {
                $table->string('location_name', 128)->nullable()->after('location_code');
            }
            if (!Schema::hasColumn('server_nodes', 'traffic_multiplier')) {
                $table->decimal('traffic_multiplier', 8, 2)->default(1)->after('traffic_used');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_enabled')) {
                $table->boolean('tcping_enabled')->default(false)->after('traffic_multiplier');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_host')) {
                $table->string('tcping_host')->nullable()->after('tcping_enabled');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_port')) {
                $table->integer('tcping_port')->nullable()->after('tcping_host');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_interval_seconds')) {
                $table->integer('tcping_interval_seconds')->default(60)->after('tcping_port');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_timeout_ms')) {
                $table->integer('tcping_timeout_ms')->default(3000)->after('tcping_interval_seconds');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_alert_after_seconds')) {
                $table->integer('tcping_alert_after_seconds')->default(300)->after('tcping_timeout_ms');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_recover_after_seconds')) {
                $table->integer('tcping_recover_after_seconds')->default(120)->after('tcping_alert_after_seconds');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_last_status')) {
                $table->string('tcping_last_status', 16)->nullable()->after('tcping_recover_after_seconds');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_last_latency_ms')) {
                $table->integer('tcping_last_latency_ms')->nullable()->after('tcping_last_status');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_last_error')) {
                $table->string('tcping_last_error', 255)->nullable()->after('tcping_last_latency_ms');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_last_sampled_at')) {
                $table->integer('tcping_last_sampled_at')->nullable()->after('tcping_last_error');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_outage_since')) {
                $table->integer('tcping_outage_since')->nullable()->after('tcping_last_sampled_at');
            }
            if (!Schema::hasColumn('server_nodes', 'tcping_recovered_since')) {
                $table->integer('tcping_recovered_since')->nullable()->after('tcping_outage_since');
            }
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('server_nodes')) {
            return;
        }

        Schema::table('server_nodes', function (Blueprint $table) {
            $columns = [
                'tcping_recovered_since',
                'tcping_outage_since',
                'tcping_last_sampled_at',
                'tcping_last_error',
                'tcping_last_latency_ms',
                'tcping_last_status',
                'tcping_recover_after_seconds',
                'tcping_alert_after_seconds',
                'tcping_timeout_ms',
                'tcping_interval_seconds',
                'tcping_port',
                'tcping_host',
                'tcping_enabled',
                'traffic_multiplier',
                'location_name',
                'location_code',
            ];

            foreach ($columns as $column) {
                if (Schema::hasColumn('server_nodes', $column)) {
                    $table->dropColumn($column);
                }
            }
        });
    }
};
