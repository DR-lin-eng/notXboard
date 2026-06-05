<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (Schema::hasTable('user_ban_records')) {
            return;
        }

        Schema::create('user_ban_records', function (Blueprint $table) {
            $table->bigIncrements('id');
            $table->integer('user_id')->comment('用户 ID');
            $table->integer('admin_id')->nullable()->comment('操作管理员 ID');
            $table->string('action', 16)->comment('ban / unban');
            $table->string('reason', 500)->comment('操作原因');
            $table->string('source', 32)->default('manual')->comment('来源');
            $table->text('context')->nullable()->comment('扩展上下文 JSON');
            $table->integer('created_at');
            $table->integer('updated_at');

            $table->index(['user_id', 'created_at'], 'idx_user_ban_records_user_created');
            $table->index(['admin_id', 'created_at'], 'idx_user_ban_records_admin_created');
            $table->index(['action', 'created_at'], 'idx_user_ban_records_action_created');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('user_ban_records');
    }
};
