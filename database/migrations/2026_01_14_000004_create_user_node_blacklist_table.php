<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('user_node_blacklist', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('node_id')->comment('节点 ID');
            $table->integer('user_id')->comment('用户 ID');
            $table->string('reason', 255)->nullable()->comment('拉黑原因');
            $table->timestamp('created_at')->useCurrent();

            $table->foreign('node_id')->references('id')->on('server_nodes')->onDelete('cascade');
            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');

            $table->unique(['node_id', 'user_id'], 'uniq_node_user_blacklist');
            $table->index(['user_id']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('user_node_blacklist');
    }
};
