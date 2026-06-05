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
        Schema::create('audit_rules', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('node_id')->comment('关联节点 ID');
            $table->enum('rule_type', ['domain', 'protocol', 'ip'])->comment('规则类型');
            $table->string('rule_pattern', 500)->comment('规则模式 (正则表达式或具体值)');
            $table->enum('action', ['block', 'allow', 'log'])->default('block')->comment('动作');
            $table->boolean('is_active')->default(true)->comment('是否启用');
            $table->timestamps();
            
            // 外键约束
            $table->foreign('node_id')->references('id')->on('server_nodes')->onDelete('cascade');
            
            // 索引
            $table->index('node_id');
            $table->index('rule_type');
            $table->index('is_active');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('audit_rules');
    }
};