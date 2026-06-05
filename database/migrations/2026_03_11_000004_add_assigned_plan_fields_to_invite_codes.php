<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_invite_code', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_invite_code', 'assigned_plan_id')) {
                $table->integer('assigned_plan_id')->nullable()->after('user_id')->comment('邀请注册赠送的套餐 ID');
            }
            if (!Schema::hasColumn('v2_invite_code', 'assigned_period')) {
                $table->string('assigned_period', 32)->nullable()->after('assigned_plan_id')->comment('邀请注册赠送的套餐周期');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_invite_code', function (Blueprint $table) {
            if (Schema::hasColumn('v2_invite_code', 'assigned_period')) {
                $table->dropColumn('assigned_period');
            }
            if (Schema::hasColumn('v2_invite_code', 'assigned_plan_id')) {
                $table->dropColumn('assigned_plan_id');
            }
        });
    }
};
