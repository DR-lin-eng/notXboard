<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_plan', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_plan', 'visibility_scope')) {
                $table->string('visibility_scope', 24)
                    ->default('public')
                    ->after('show')
                    ->comment('public|link_only|assigned_only');
                $table->index('visibility_scope');
            }

            if (!Schema::hasColumn('v2_plan', 'access_user_ids')) {
                $table->json('access_user_ids')
                    ->nullable()
                    ->after('visibility_scope')
                    ->comment('assigned users for this plan');
            }

            if (!Schema::hasColumn('v2_plan', 'share_token')) {
                $table->string('share_token', 64)
                    ->nullable()
                    ->after('access_user_ids')
                    ->comment('dedicated share token');
                $table->unique('share_token');
            }

            if (!Schema::hasColumn('v2_plan', 'share_discount_type')) {
                $table->unsignedTinyInteger('share_discount_type')
                    ->default(0)
                    ->after('share_token')
                    ->comment('0:none,1:fixed,2:percent');
            }

            if (!Schema::hasColumn('v2_plan', 'share_discount_value')) {
                $table->unsignedInteger('share_discount_value')
                    ->default(0)
                    ->after('share_discount_type')
                    ->comment('fixed:fen,percent:1-100');
            }
        });

        Schema::table('v2_coupon', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_coupon', 'owner_user_id')) {
                $table->integer('owner_user_id')
                    ->nullable()
                    ->after('name')
                    ->comment('coupon publisher user id');
                $table->index('owner_user_id');
            }

            if (!Schema::hasColumn('v2_coupon', 'source_plan_id')) {
                $table->integer('source_plan_id')
                    ->nullable()
                    ->after('owner_user_id')
                    ->comment('plan id that owns this coupon');
                $table->index('source_plan_id');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_coupon', function (Blueprint $table) {
            if (Schema::hasColumn('v2_coupon', 'source_plan_id')) {
                $table->dropIndex(['source_plan_id']);
                $table->dropColumn('source_plan_id');
            }

            if (Schema::hasColumn('v2_coupon', 'owner_user_id')) {
                $table->dropIndex(['owner_user_id']);
                $table->dropColumn('owner_user_id');
            }
        });

        Schema::table('v2_plan', function (Blueprint $table) {
            if (Schema::hasColumn('v2_plan', 'share_discount_value')) {
                $table->dropColumn('share_discount_value');
            }

            if (Schema::hasColumn('v2_plan', 'share_discount_type')) {
                $table->dropColumn('share_discount_type');
            }

            if (Schema::hasColumn('v2_plan', 'share_token')) {
                $table->dropUnique('v2_plan_share_token_unique');
                $table->dropColumn('share_token');
            }

            if (Schema::hasColumn('v2_plan', 'access_user_ids')) {
                $table->dropColumn('access_user_ids');
            }

            if (Schema::hasColumn('v2_plan', 'visibility_scope')) {
                $table->dropIndex(['visibility_scope']);
                $table->dropColumn('visibility_scope');
            }
        });
    }
};

