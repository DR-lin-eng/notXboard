<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('sponsor_donations', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('user_id')->nullable()->comment('用户ID（可为空）');
            $table->string('trade_no', 36)->unique();
            $table->integer('total_amount')->comment('金额（分）');
            $table->integer('payment_id')->nullable();
            $table->string('callback_no')->nullable();
            $table->integer('status')->default(0)->comment('0待支付 1已完成 2已取消');
            $table->integer('paid_at')->nullable();
            $table->timestamps();

            $table->index(['user_id']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('sponsor_donations');
    }
};
