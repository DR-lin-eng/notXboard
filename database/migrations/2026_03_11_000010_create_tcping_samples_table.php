<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (Schema::hasTable('tcping_samples')) {
            return;
        }

        Schema::create('tcping_samples', function (Blueprint $table) {
            $table->bigIncrements('id');
            $table->unsignedBigInteger('node_id');
            $table->unsignedBigInteger('agent_id')->nullable();
            $table->boolean('is_reachable')->default(false);
            $table->integer('latency_ms')->nullable();
            $table->boolean('is_timeout')->default(false);
            $table->string('error_message', 255)->nullable();
            $table->integer('sampled_at');
            $table->integer('created_at');
            $table->integer('updated_at');

            $table->index(['node_id', 'sampled_at'], 'idx_tcping_samples_node_sampled');
            $table->index(['agent_id', 'sampled_at'], 'idx_tcping_samples_agent_sampled');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('tcping_samples');
    }
};
