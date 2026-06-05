<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('tcping_agents')) {
            return;
        }

        Schema::table('tcping_agents', function (Blueprint $table) {
            if (!Schema::hasColumn('tcping_agents', 'location_code')) {
                $table->string('location_code', 16)->nullable()->after('name');
            }
            if (!Schema::hasColumn('tcping_agents', 'location_name')) {
                $table->string('location_name', 128)->nullable()->after('location_code');
            }
            if (!Schema::hasColumn('tcping_agents', 'location_province')) {
                // For CN mainland probes, require province name (e.g. 广东/上海) in the app layer.
                $table->string('location_province', 64)->nullable()->after('location_name');
            }
        });
    }

    public function down(): void
    {
        if (!Schema::hasTable('tcping_agents')) {
            return;
        }

        Schema::table('tcping_agents', function (Blueprint $table) {
            $columns = ['location_province', 'location_name', 'location_code'];
            foreach ($columns as $column) {
                if (Schema::hasColumn('tcping_agents', $column)) {
                    $table->dropColumn($column);
                }
            }
        });
    }
};

