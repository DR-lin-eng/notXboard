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
        Schema::create('user_individual_limits', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('用户 ID');
            $table->integer('speed_limit_up')->default(0)->comment('上传限速 (Mbps, 0表示使用组配置)');
            $table->integer('speed_limit_down')->default(0)->comment('下载限速 (Mbps, 0表示使用组配置)');
            $table->integer('device_limit')->default(0)->comment('设备数限制 (0表示使用组配置)');
            $table->integer('connection_limit')->default(0)->comment('连接数限制 (0表示使用组配置)');
            $table->timestamps();
            
            // 外键约束
            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            
            // 唯一约束
            $table->unique('user_id');
            
            // 索引
            $table->index('user_id');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('user_individual_limits');
    }
};
