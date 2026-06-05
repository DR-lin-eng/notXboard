<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (Schema::hasTable('user_risk_reviews')) {
            return;
        }

        Schema::create('user_risk_reviews', function (Blueprint $table) {
            $table->bigIncrements('id');
            $table->integer('user_id')->comment('用户 ID');
            $table->string('source', 32)->default('shared_ip')->comment('风险来源');
            $table->string('shared_ip', 45)->comment('命中的共享 IP');
            $table->integer('matched_user_count')->default(0)->comment('共享 IP 命中的用户数');
            $table->text('matched_user_ids')->nullable()->comment('命中的用户 ID 列表 JSON');
            $table->string('risk_level', 16)->default('medium')->comment('风险等级');
            $table->unsignedInteger('suspicion_score')->default(0)->comment('疑似滥用评分');
            $table->string('llm_model', 255)->nullable()->comment('审查使用的 LLM 模型');
            $table->text('summary')->nullable()->comment('审查摘要');
            $table->text('recommendation')->nullable()->comment('处理建议');
            $table->mediumText('raw_response')->nullable()->comment('LLM 原始响应');
            $table->mediumText('evidence')->nullable()->comment('证据 JSON');
            $table->integer('reviewed_at')->comment('审查时间');
            $table->integer('created_at');
            $table->integer('updated_at');

            $table->index(['user_id', 'reviewed_at'], 'idx_user_risk_reviews_user_reviewed');
            $table->index(['shared_ip', 'reviewed_at'], 'idx_user_risk_reviews_ip_reviewed');
            $table->index(['risk_level', 'reviewed_at'], 'idx_user_risk_reviews_level_reviewed');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('user_risk_reviews');
    }
};
