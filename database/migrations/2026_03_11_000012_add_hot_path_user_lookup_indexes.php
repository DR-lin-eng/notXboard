<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            $table->index('token', 'idx_v2_user_token');
            $table->index('subscribe_path', 'idx_v2_user_subscribe_path');
        });
    }

    public function down(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            $table->dropIndex('idx_v2_user_token');
            $table->dropIndex('idx_v2_user_subscribe_path');
        });
    }
};
