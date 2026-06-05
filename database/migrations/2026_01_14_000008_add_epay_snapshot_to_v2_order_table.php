<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_order', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_order', 'epay_pid')) {
                $table->string('epay_pid', 64)->nullable()->after('callback_no')->comment('EPay pid snapshot');
            }
            if (!Schema::hasColumn('v2_order', 'epay_url')) {
                $table->string('epay_url', 255)->nullable()->after('epay_pid')->comment('EPay gateway url snapshot');
            }
            if (!Schema::hasColumn('v2_order', 'epay_key_encrypted')) {
                $table->text('epay_key_encrypted')->nullable()->after('epay_url')->comment('EPay key snapshot (encrypted)');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_order', function (Blueprint $table) {
            if (Schema::hasColumn('v2_order', 'epay_key_encrypted')) {
                $table->dropColumn('epay_key_encrypted');
            }
            if (Schema::hasColumn('v2_order', 'epay_url')) {
                $table->dropColumn('epay_url');
            }
            if (Schema::hasColumn('v2_order', 'epay_pid')) {
                $table->dropColumn('epay_pid');
            }
        });
    }
};

