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
        Schema::create('user_group_limits', function (Blueprint $table) {
            $table->id();
            $table->tinyInteger('trust_level')->comment('信任等级 (0-4)');
            $table->integer('speed_limit_up')->default(0)->comment('上传限速 (Mbps)');
            $table->integer('speed_limit_down')->default(0)->comment('下载限速 (Mbps)');
            $table->integer('device_limit')->default(0)->comment('设备数限制');
            $table->integer('connection_limit')->default(0)->comment('连接数限制');
            $table->timestamps();
            
            // 唯一约束
            $table->unique('trust_level');
            
            // 索引
            $table->index('trust_level');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('user_group_limits');
    }
};