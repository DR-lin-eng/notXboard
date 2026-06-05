<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::table('server_nodes', function (Blueprint $table) {
            if (!Schema::hasColumn('server_nodes', 'service_port')) {
                $table->integer('service_port')
                    ->nullable()
                    ->after('port')
                    ->comment('服务端口(下发给节点，空则等于访问端口)');
            }
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('server_nodes', function (Blueprint $table) {
            if (Schema::hasColumn('server_nodes', 'service_port')) {
                $table->dropColumn('service_port');
            }
        });
    }
};

