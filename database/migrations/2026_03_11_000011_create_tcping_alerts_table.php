<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (Schema::hasTable('tcping_alerts')) {
            return;
        }

        Schema::create('tcping_alerts', function (Blueprint $table) {
            $table->bigIncrements('id');
            $table->unsignedBigInteger('node_id');
            $table->integer('user_id');
            $table->string('status', 16)->default('active');
            $table->integer('started_at');
            $table->integer('triggered_at');
            $table->integer('recovered_at')->nullable();
            $table->string('latest_error', 255)->nullable();
            $table->integer('created_at');
            $table->integer('updated_at');

            $table->index(['user_id', 'status'], 'idx_tcping_alerts_user_status');
            $table->index(['node_id', 'status'], 'idx_tcping_alerts_node_status');
            $table->index(['triggered_at'], 'idx_tcping_alerts_triggered');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('tcping_alerts');
    }
};
