<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('v2_user')) {
            return;
        }

        Schema::table('v2_user', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_user', 'subscription_credential_version')) {
                $table->integer('subscription_credential_version')->default(0)->after('subscribe_salt');
            }
            if (!Schema::hasColumn('v2_user', 'last_subscription_credential_rotation_at')) {
                $table->integer('last_subscription_credential_rotation_at')->nullable()->after('subscription_credential_version');
            }
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('v2_user')) {
            return;
        }

        Schema::table('v2_user', function (Blueprint $table) {
            if (Schema::hasColumn('v2_user', 'last_subscription_credential_rotation_at')) {
                $table->dropColumn('last_subscription_credential_rotation_at');
            }
            if (Schema::hasColumn('v2_user', 'subscription_credential_version')) {
                $table->dropColumn('subscription_credential_version');
            }
        });
    }
};
