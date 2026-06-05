<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (Schema::hasTable('tcping_agents')) {
            return;
        }

        Schema::create('tcping_agents', function (Blueprint $table) {
            $table->bigIncrements('id');
            $table->integer('user_id');
            $table->string('name', 128);
            $table->string('token', 96)->unique();
            $table->boolean('is_enabled')->default(true);
            $table->integer('last_heartbeat_at')->nullable();
            $table->integer('last_sync_at')->nullable();
            $table->integer('created_at');
            $table->integer('updated_at');

            $table->index(['user_id', 'is_enabled'], 'idx_tcping_agents_user_enabled');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('tcping_agents');
    }
};
