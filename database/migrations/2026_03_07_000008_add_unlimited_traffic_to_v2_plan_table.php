<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('v2_plan')) {
            return;
        }

        Schema::table('v2_plan', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_plan', 'is_unlimited_traffic')) {
                $table->boolean('is_unlimited_traffic')
                    ->default(false)
                    ->after('transfer_enable')
                    ->comment('是否无限流量');
            }
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('v2_plan')) {
            return;
        }

        Schema::table('v2_plan', function (Blueprint $table) {
            if (Schema::hasColumn('v2_plan', 'is_unlimited_traffic')) {
                $table->dropColumn('is_unlimited_traffic');
            }
        });
    }
};

