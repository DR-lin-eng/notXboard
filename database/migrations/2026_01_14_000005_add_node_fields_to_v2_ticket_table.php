<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_ticket', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_ticket', 'node_id')) {
                $table->unsignedBigInteger('node_id')->nullable()->after('user_id')->comment('server_nodes id');
                $table->index('node_id');
            }
            if (!Schema::hasColumn('v2_ticket', 'assigned_admin_user_id')) {
                $table->unsignedBigInteger('assigned_admin_user_id')->nullable()->after('node_id')->comment('responsible admin user id');
                $table->index('assigned_admin_user_id');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_ticket', function (Blueprint $table) {
            if (Schema::hasColumn('v2_ticket', 'assigned_admin_user_id')) {
                $table->dropIndex(['assigned_admin_user_id']);
                $table->dropColumn('assigned_admin_user_id');
            }
            if (Schema::hasColumn('v2_ticket', 'node_id')) {
                $table->dropIndex(['node_id']);
                $table->dropColumn('node_id');
            }
        });
    }
};

