<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_plan', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_plan', 'min_trust_level')) {
                $table->unsignedTinyInteger('min_trust_level')->nullable()->after('owner_user_id')->comment('Minimum Linux DO trust_level required to view/buy');
            }
            if (!Schema::hasColumn('v2_plan', 'free_quota_gb_by_trust_level')) {
                $table->json('free_quota_gb_by_trust_level')->nullable()->after('min_trust_level')->comment('Free quota per trust_level (GB/month)');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_plan', function (Blueprint $table) {
            if (Schema::hasColumn('v2_plan', 'free_quota_gb_by_trust_level')) {
                $table->dropColumn('free_quota_gb_by_trust_level');
            }
            if (Schema::hasColumn('v2_plan', 'min_trust_level')) {
                $table->dropColumn('min_trust_level');
            }
        });
    }
};

