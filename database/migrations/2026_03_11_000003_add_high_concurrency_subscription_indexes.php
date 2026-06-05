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
            $table->index(
                ['user_id', 'plan_id', 'status', 'expired_at'],
                'idx_ups_user_plan_status_expired'
            );
            $table->index(
                ['plan_id', 'user_id', 'status', 'expired_at'],
                'idx_ups_plan_user_status_expired'
            );
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('user_plan_subscriptions')) {
            return;
        }

        Schema::table('user_plan_subscriptions', function (Blueprint $table) {
            $table->dropIndex('idx_ups_user_plan_status_expired');
            $table->dropIndex('idx_ups_plan_user_status_expired');
        });
    }
};
