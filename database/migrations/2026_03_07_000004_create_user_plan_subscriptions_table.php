<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('user_plan_subscriptions', function (Blueprint $table) {
            $table->bigIncrements('id');
            $table->integer('user_id')->comment('用户 ID');
            $table->integer('plan_id')->comment('套餐 ID');
            $table->integer('order_id')->unique()->comment('来源订单 ID');
            $table->string('period', 32)->comment('购买周期');
            $table->bigInteger('traffic_allowance_kb')->default(0)->comment('该实例总流量额度(KB)');
            $table->bigInteger('used_traffic_kb')->default(0)->comment('已使用流量(KB)');
            $table->integer('started_at')->comment('生效时间');
            $table->integer('expired_at')->nullable()->comment('到期时间，NULL 表示长期');
            $table->tinyInteger('status')->default(1)->comment('1=active,2=expired,3=revoked');
            $table->integer('created_at');
            $table->integer('updated_at');

            $table->index(['user_id', 'status', 'expired_at'], 'idx_user_status_expired');
            $table->index(['plan_id', 'status', 'expired_at'], 'idx_plan_status_expired');
            $table->index(['user_id', 'plan_id', 'status'], 'idx_user_plan_status');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('user_plan_subscriptions');
    }
};
