<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('server_nodes', function (Blueprint $table) {
            $table->index('v2bx_token', 'idx_server_nodes_v2bx_token');
            $table->index(['v2bx_node_id', 'v2bx_token'], 'idx_server_nodes_v2bx_node_token');
            $table->index(['status', 'user_id'], 'idx_server_nodes_status_user');
        });
    }

    public function down(): void
    {
        Schema::table('server_nodes', function (Blueprint $table) {
            $table->dropIndex('idx_server_nodes_v2bx_token');
            $table->dropIndex('idx_server_nodes_v2bx_node_token');
            $table->dropIndex('idx_server_nodes_status_user');
        });
    }
};
