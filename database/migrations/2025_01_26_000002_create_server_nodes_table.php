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
        Schema::create('server_nodes', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('服务器提供者 ID');
            $table->string('name')->comment('节点名称');
            $table->string('host')->comment('服务器地址');
            $table->integer('port')->comment('端口');
            $table->integer('service_port')->nullable()->comment('服务端口(下发给节点，空则等于访问端口)');
            $table->string('protocol', 32)->comment('协议类型');
            $table->json('settings')->nullable()->comment('节点配置');
            $table->unsignedBigInteger('traffic_limit')->default(0)->comment('流量限制 (KB)');
            $table->unsignedBigInteger('traffic_used')->default(0)->comment('已使用流量 (KB)');
            $table->json('access_control')->nullable()->comment('访问控制配置');
            $table->enum('status', ['active', 'inactive', 'maintenance', 'deploying'])->default('inactive')->comment('节点状态');
            
            // V2bX 集成字段
            $table->integer('v2bx_node_id')->nullable()->comment('V2bX 节点 ID');
            $table->json('v2bx_config')->nullable()->comment('V2bX 配置');
            $table->string('v2bx_token')->nullable()->comment('V2bX 认证令牌');
            
            // 限制配置
            $table->integer('device_limit')->default(0)->comment('设备数限制');
            $table->integer('connection_limit')->default(0)->comment('连接数限制');
            $table->integer('speed_limit_up')->default(0)->comment('上传限速 (Mbps)');
            $table->integer('speed_limit_down')->default(0)->comment('下载限速 (Mbps)');
            $table->integer('cross_node_ip_limit')->default(0)->comment('跨节点IP限制');
            $table->integer('concurrent_ip_limit')->default(0)->comment('跨节点并发IP限制 (仅超管可配置)');
            
            $table->timestamps();
            
            // 外键约束
            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            
            // 索引
            $table->index('user_id');
            $table->index('status');
            $table->index('protocol');
            $table->index('v2bx_node_id');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('server_nodes');
    }
};
