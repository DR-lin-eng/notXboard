<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('user_plan_subscriptions')) {
            return;
        }

        Schema::table('user_plan_subscriptions', function (Blueprint $table) {
            if (!Schema::hasColumn('user_plan_subscriptions', 'used_traffic_kb')) {
                $table->bigInteger('used_traffic_kb')
                    ->default(0)
                    ->after('traffic_allowance_kb')
                    ->comment('已使用流量(KB)');
            }
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('user_plan_subscriptions')) {
            return;
        }

        Schema::table('user_plan_subscriptions', function (Blueprint $table) {
            if (Schema::hasColumn('user_plan_subscriptions', 'used_traffic_kb')) {
                $table->dropColumn('used_traffic_kb');
            }
        });
    }
};

