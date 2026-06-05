<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('v2_ticket') || Schema::hasColumn('v2_ticket', 'last_reply_user_id')) {
            return;
        }

        Schema::table('v2_ticket', function (Blueprint $table) {
            $table->integer('last_reply_user_id')->nullable();
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('v2_ticket') || !Schema::hasColumn('v2_ticket', 'last_reply_user_id')) {
            return;
        }

        Schema::table('v2_ticket', function (Blueprint $table) {
            $table->dropColumn('last_reply_user_id');
        });
    }
};
