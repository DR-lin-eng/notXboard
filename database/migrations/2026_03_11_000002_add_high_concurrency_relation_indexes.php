<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('user_node_plan_access', function (Blueprint $table) {
            $table->index(['user_id', 'plan_id', 'node_id'], 'idx_user_node_plan_access_user_plan_node');
            $table->index(['node_id', 'plan_id', 'user_id'], 'idx_user_node_plan_access_node_plan_user');
        });

        Schema::table('node_traffic_records', function (Blueprint $table) {
            $table->index(['node_id', 'record_date'], 'idx_node_traffic_records_node_date');
            $table->index(['user_id', 'record_date'], 'idx_node_traffic_records_user_date');
        });
    }

    public function down(): void
    {
        Schema::table('user_node_plan_access', function (Blueprint $table) {
            $table->dropIndex('idx_user_node_plan_access_user_plan_node');
            $table->dropIndex('idx_user_node_plan_access_node_plan_user');
        });

        Schema::table('node_traffic_records', function (Blueprint $table) {
            $table->dropIndex('idx_node_traffic_records_node_date');
            $table->dropIndex('idx_node_traffic_records_user_date');
        });
    }
};
