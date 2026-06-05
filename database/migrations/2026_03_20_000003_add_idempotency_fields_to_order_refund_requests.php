<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (Schema::hasTable('order_refund_requests')) {
            Schema::table('order_refund_requests', function (Blueprint $table) {
                if (!Schema::hasColumn('order_refund_requests', 'balance_refunded_amount')) {
                    $table->integer('balance_refunded_amount')->default(0)->after('charged_amount');
                }
                if (!Schema::hasColumn('order_refund_requests', 'gateway_refunded_amount')) {
                    $table->integer('gateway_refunded_amount')->default(0)->after('balance_refunded_amount');
                }
                if (!Schema::hasColumn('order_refund_requests', 'site_balance_fallback_amount')) {
                    $table->integer('site_balance_fallback_amount')->default(0)->after('gateway_refunded_amount');
                }
                if (!Schema::hasColumn('order_refund_requests', 'gateway_refund_pending_amount')) {
                    $table->integer('gateway_refund_pending_amount')->default(0)->after('site_balance_fallback_amount');
                }
                if (!Schema::hasColumn('order_refund_requests', 'gateway_refund_started_at')) {
                    $table->timestamp('gateway_refund_started_at')->nullable()->after('gateway_refund_pending_amount');
                }
                if (!Schema::hasColumn('order_refund_requests', 'refunded_at')) {
                    $table->timestamp('refunded_at')->nullable()->after('voting_ends_at');
                }
            });
        }
    }

    public function down(): void
    {
        if (Schema::hasTable('order_refund_requests')) {
            Schema::table('order_refund_requests', function (Blueprint $table) {
                if (Schema::hasColumn('order_refund_requests', 'refunded_at')) {
                    $table->dropColumn('refunded_at');
                }
                if (Schema::hasColumn('order_refund_requests', 'gateway_refund_started_at')) {
                    $table->dropColumn('gateway_refund_started_at');
                }
                if (Schema::hasColumn('order_refund_requests', 'gateway_refund_pending_amount')) {
                    $table->dropColumn('gateway_refund_pending_amount');
                }
                if (Schema::hasColumn('order_refund_requests', 'site_balance_fallback_amount')) {
                    $table->dropColumn('site_balance_fallback_amount');
                }
                if (Schema::hasColumn('order_refund_requests', 'gateway_refunded_amount')) {
                    $table->dropColumn('gateway_refunded_amount');
                }
                if (Schema::hasColumn('order_refund_requests', 'balance_refunded_amount')) {
                    $table->dropColumn('balance_refunded_amount');
                }
            });
        }
    }
};
