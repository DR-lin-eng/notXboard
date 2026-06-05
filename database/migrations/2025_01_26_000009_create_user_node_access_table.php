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
        Schema::create('user_node_access', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('用户 ID');
            $table->unsignedBigInteger('node_id')->comment('节点 ID');
            $table->enum('access_type', ['individual', 'group'])->comment('访问类型');
            $table->timestamp('granted_at')->useCurrent()->comment('授权时间');
            
            // 外键约束
            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            $table->foreign('node_id')->references('id')->on('server_nodes')->onDelete('cascade');
            
            // 索引
            $table->index(['node_id', 'user_id'], 'idx_user_node_access_node_user');

            // 唯一约束
            $table->unique(['user_id', 'node_id'], 'unique_user_node');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('user_node_access');
    }
};
