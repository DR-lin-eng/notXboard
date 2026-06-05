<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('v2_notice')) {
            return;
        }

        Schema::table('v2_notice', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_notice', 'popup')) {
                $table->boolean('popup')->default(false)->after('show');
            }
            if (!Schema::hasColumn('v2_notice', 'author_user_id')) {
                $table->integer('author_user_id')->nullable()->after('popup');
            }
            if (!Schema::hasColumn('v2_notice', 'scope_type')) {
                $table->string('scope_type', 32)->default('global')->after('author_user_id');
            }
            if (!Schema::hasColumn('v2_notice', 'target_plan_ids')) {
                $table->text('target_plan_ids')->nullable()->after('scope_type');
            }
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('v2_notice')) {
            return;
        }

        Schema::table('v2_notice', function (Blueprint $table) {
            if (Schema::hasColumn('v2_notice', 'target_plan_ids')) {
                $table->dropColumn('target_plan_ids');
            }
            if (Schema::hasColumn('v2_notice', 'scope_type')) {
                $table->dropColumn('scope_type');
            }
            if (Schema::hasColumn('v2_notice', 'author_user_id')) {
                $table->dropColumn('author_user_id');
            }
            if (Schema::hasColumn('v2_notice', 'popup')) {
                $table->dropColumn('popup');
            }
        });
    }
};
