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
        Schema::create('node_traffic_records', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('用户 ID');
            $table->unsignedBigInteger('node_id')->comment('节点 ID');
            $table->unsignedBigInteger('upload_traffic')->default(0)->comment('上传流量 (KB)');
            $table->unsignedBigInteger('download_traffic')->default(0)->comment('下载流量 (KB)');
            $table->date('record_date')->comment('记录日期');
            $table->timestamps();
            
            // 外键约束
            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            $table->foreign('node_id')->references('id')->on('server_nodes')->onDelete('cascade');
            
            // 唯一约束
            $table->unique(['user_id', 'node_id', 'record_date'], 'unique_user_node_date');
            
            // 索引
            $table->index('user_id');
            $table->index('node_id');
            $table->index('record_date');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('node_traffic_records');
    }
};
