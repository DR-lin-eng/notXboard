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
        if (!Schema::hasTable('server_nodes')) {
            return;
        }

        Schema::table('server_nodes', function (Blueprint $table) {
            $table->string('protocol', 32)->comment('协议类型')->change();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        // Intentionally left as no-op.
        // Existing rows may already contain newer protocol values.
    }
};
