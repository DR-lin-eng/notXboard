<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_user', 'ban_reason')) {
                $table->string('ban_reason', 500)->nullable()->after('banned')->comment('当前封禁原因');
            }

            if (!Schema::hasColumn('v2_user', 'banned_at')) {
                $table->integer('banned_at')->nullable()->after('ban_reason')->comment('封禁时间');
            }

            if (!Schema::hasColumn('v2_user', 'banned_by_admin_id')) {
                $table->integer('banned_by_admin_id')->nullable()->after('banned_at')->comment('封禁操作管理员 ID');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            foreach (['banned_by_admin_id', 'banned_at', 'ban_reason'] as $column) {
                if (Schema::hasColumn('v2_user', $column)) {
                    $table->dropColumn($column);
                }
            }
        });
    }
};
