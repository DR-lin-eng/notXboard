<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_plan', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_plan', 'scope')) {
                $table->string('scope', 20)->default('legacy')->after('name')->comment('legacy|node');
                $table->index('scope');
            }
            if (!Schema::hasColumn('v2_plan', 'owner_user_id')) {
                $table->unsignedBigInteger('owner_user_id')->nullable()->after('scope')->comment('node plan publisher user id');
                $table->index('owner_user_id');
            }
            if (!Schema::hasColumn('v2_plan', 'node_ids')) {
                $table->json('node_ids')->nullable()->after('owner_user_id')->comment('server_nodes ids for node plan');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_plan', function (Blueprint $table) {
            if (Schema::hasColumn('v2_plan', 'node_ids')) {
                $table->dropColumn('node_ids');
            }
            if (Schema::hasColumn('v2_plan', 'owner_user_id')) {
                $table->dropIndex(['owner_user_id']);
                $table->dropColumn('owner_user_id');
            }
            if (Schema::hasColumn('v2_plan', 'scope')) {
                $table->dropIndex(['scope']);
                $table->dropColumn('scope');
            }
        });
    }
};

