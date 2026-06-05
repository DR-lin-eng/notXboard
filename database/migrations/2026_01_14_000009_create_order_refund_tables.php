<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('order_refund_requests', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('order_id');
            $table->string('trade_no', 36);
            $table->unsignedBigInteger('user_id');
            $table->unsignedBigInteger('plan_id');
            $table->unsignedBigInteger('assigned_admin_user_id')->nullable();

            $table->string('status', 20)->default('pending'); // pending|voting|approved|denied|refunded|failed
            $table->text('reason')->nullable();

            $table->integer('gateway_amount')->comment('EPay original amount (cents)')->default(0);
            $table->string('gateway_trade_no', 64)->nullable()->comment('EPay trade_no (gateway)');
            $table->string('epay_pid', 64)->nullable();
            $table->string('epay_url', 255)->nullable();
            $table->text('epay_key_encrypted')->nullable();

            $table->bigInteger('used_kb')->nullable();
            $table->bigInteger('allowance_kb')->nullable();
            $table->integer('refund_amount')->nullable()->comment('Refund to user (cents)');
            $table->integer('charged_amount')->nullable()->comment('Fee charged for used traffic (cents)');

            $table->timestamp('voting_ends_at')->nullable();
            $table->timestamp('resolved_at')->nullable();
            $table->unsignedBigInteger('resolved_by_user_id')->nullable();
            $table->string('decision', 20)->nullable(); // approve|deny

            $table->timestamps();

            $table->unique(['order_id'], 'uniq_order_refund');
            $table->index(['user_id', 'status']);
            $table->index(['assigned_admin_user_id', 'status']);
        });

        Schema::create('order_refund_evidences', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('refund_request_id');
            $table->unsignedBigInteger('user_id');
            $table->string('role', 20)->comment('user|admin|member|super_admin');
            $table->text('content');
            $table->timestamps();

            $table->foreign('refund_request_id')->references('id')->on('order_refund_requests')->onDelete('cascade');
            $table->index(['refund_request_id']);
        });

        Schema::create('order_refund_votes', function (Blueprint $table) {
            $table->id();
            $table->unsignedBigInteger('refund_request_id');
            $table->unsignedBigInteger('user_id');
            $table->string('vote', 10)->comment('approve|deny');
            $table->timestamps();

            $table->foreign('refund_request_id')->references('id')->on('order_refund_requests')->onDelete('cascade');
            $table->unique(['refund_request_id', 'user_id'], 'uniq_refund_vote');
            $table->index(['refund_request_id']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('order_refund_votes');
        Schema::dropIfExists('order_refund_evidences');
        Schema::dropIfExists('order_refund_requests');
    }
};

