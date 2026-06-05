<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('user_node_plan_access', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('用户 ID');
            $table->unsignedBigInteger('node_id')->comment('节点 ID');
            $table->integer('plan_id')->comment('套餐 ID');
            $table->timestamp('granted_at')->useCurrent()->comment('授权时间');

            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            $table->foreign('node_id')->references('id')->on('server_nodes')->onDelete('cascade');
            $table->foreign('plan_id')->references('id')->on('v2_plan')->onDelete('cascade');

            $table->unique(['user_id', 'node_id', 'plan_id'], 'uniq_user_node_plan');
            $table->index(['user_id', 'node_id']);
            $table->index(['plan_id']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('user_node_plan_access');
    }
};
