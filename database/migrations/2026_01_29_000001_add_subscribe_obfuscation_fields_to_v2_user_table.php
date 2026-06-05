<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            $table->string('subscribe_path', 32)->nullable()->after('token');
            $table->string('subscribe_key', 32)->nullable()->after('subscribe_path');
            $table->string('subscribe_salt', 32)->nullable()->after('subscribe_key');
        });
    }

    public function down(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            $table->dropColumn(['subscribe_path', 'subscribe_key', 'subscribe_salt']);
        });
    }
};
