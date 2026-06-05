<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            if (!Schema::hasColumn('v2_user', 'concurrent_ip_limit')) {
                $table->unsignedInteger('concurrent_ip_limit')
                    ->default(0)
                    ->after('device_limit')
                    ->comment('跨节点并发IP限制（0表示默认策略）');
                $table->index('concurrent_ip_limit');
            }
        });
    }

    public function down(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            if (Schema::hasColumn('v2_user', 'concurrent_ip_limit')) {
                $table->dropIndex(['concurrent_ip_limit']);
                $table->dropColumn('concurrent_ip_limit');
            }
        });
    }
};

