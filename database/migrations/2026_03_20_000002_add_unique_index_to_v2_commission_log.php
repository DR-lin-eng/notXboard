<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('v2_commission_log')) {
            return;
        }

        try {
            Schema::table('v2_commission_log', function (Blueprint $table) {
                $table->unique(
                    ['invite_user_id', 'user_id', 'trade_no'],
                    'uniq_v2_commission_log_trade_inviter_user'
                );
            });
        } catch (\Throwable $e) {
            // Ignore when the index already exists or current data must be cleaned first.
        }
    }

    public function down(): void
    {
        if (!Schema::hasTable('v2_commission_log')) {
            return;
        }

        try {
            Schema::table('v2_commission_log', function (Blueprint $table) {
                $table->dropUnique('uniq_v2_commission_log_trade_inviter_user');
            });
        } catch (\Throwable $e) {
            // Ignore when the index does not exist.
        }
    }
};
