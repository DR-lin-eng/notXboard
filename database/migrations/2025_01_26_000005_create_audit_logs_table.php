<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('audit_logs', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('用户 ID');
            $table->unsignedBigInteger('node_id')->comment('节点 ID');
            $table->unsignedBigInteger('rule_id')->nullable()->comment('触发的规则 ID');
            $table->string('ip_address', 45)->comment('源 IP 地址');
            $table->string('target_domain')->nullable()->comment('目标域名');
            $table->string('target_protocol', 50)->nullable()->comment('目标协议');
            $table->enum('action_taken', ['blocked', 'allowed', 'logged'])->comment('执行的动作');
            $table->timestamps();
            
            // 外键约束
            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            $table->foreign('node_id')->references('id')->on('server_nodes')->onDelete('cascade');
            $table->foreign('rule_id')->references('id')->on('audit_rules')->onDelete('set null');
            
            // 索引
            $table->index('user_id');
            $table->index('node_id');
            $table->index('created_at');
            $table->index('action_taken');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('audit_logs');
    }
};
