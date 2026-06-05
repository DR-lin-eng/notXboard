<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasColumn('v2_user', 'concurrent_ip_limit')) {
            return;
        }

        Schema::table('v2_user', function (Blueprint $table) {
            $table->unsignedInteger('concurrent_ip_limit')
                ->default(3)
                ->comment('同用户多IP并发限制')
                ->change();
        });
    }

    public function down(): void
    {
        if (!Schema::hasColumn('v2_user', 'concurrent_ip_limit')) {
            return;
        }

        Schema::table('v2_user', function (Blueprint $table) {
            $table->unsignedInteger('concurrent_ip_limit')
                ->default(0)
                ->comment('跨节点并发IP限制（0表示默认策略）')
                ->change();
        });
    }
};
