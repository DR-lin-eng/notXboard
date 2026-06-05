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
        Schema::create('user_online_sessions', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('用户 ID');
            $table->unsignedBigInteger('node_id')->comment('节点 ID');
            $table->string('ip_address', 45)->comment('IP 地址');
            $table->text('user_agent')->nullable()->comment('用户代理');
            $table->integer('connection_count')->default(1)->comment('连接数');
            $table->unsignedBigInteger('upload_traffic')->default(0)->comment('上传流量 (KB)');
            $table->unsignedBigInteger('download_traffic')->default(0)->comment('下载流量 (KB)');
            $table->timestamp('last_activity')->useCurrent()->useCurrentOnUpdate()->comment('最后活动时间');
            $table->timestamps();
            
            // 外键约束
            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            $table->foreign('node_id')->references('id')->on('server_nodes')->onDelete('cascade');
            
            // 索引
            $table->index('user_id');
            $table->index('node_id');
            $table->index('ip_address');
            $table->index('last_activity');
            $table->index(['user_id', 'last_activity', 'ip_address'], 'idx_user_online_sessions_user_last_activity_ip');
            
            // 唯一约束
            $table->unique(['user_id', 'node_id', 'ip_address'], 'unique_user_node_ip');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('user_online_sessions');
    }
};
