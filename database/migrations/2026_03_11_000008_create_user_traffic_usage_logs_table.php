<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (Schema::hasTable('user_traffic_usage_logs')) {
            return;
        }

        Schema::create('user_traffic_usage_logs', function (Blueprint $table) {
            $table->bigIncrements('id');
            $table->integer('user_id');
            $table->unsignedBigInteger('node_id');
            $table->unsignedBigInteger('raw_traffic_kb')->default(0);
            $table->unsignedBigInteger('billed_traffic_kb')->default(0);
            $table->decimal('multiplier_snapshot', 8, 2)->default(1);
            $table->string('source', 32)->default('push');
            $table->integer('recorded_at');
            $table->integer('created_at');
            $table->integer('updated_at');

            $table->index(['user_id', 'recorded_at'], 'idx_utul_user_recorded');
            $table->index(['node_id', 'recorded_at'], 'idx_utul_node_recorded');
            $table->index(['recorded_at'], 'idx_utul_recorded');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('user_traffic_usage_logs');
    }
};
