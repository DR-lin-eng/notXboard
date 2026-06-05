<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('user_payment_profiles', function (Blueprint $table) {
            $table->id();
            $table->integer('user_id')->comment('用户ID');
            $table->string('provider', 32)->comment('支付协议/提供商，例如 epay');
            $table->string('pid', 64)->nullable()->comment('Client ID');
            $table->text('key_encrypted')->nullable()->comment('Client Secret (encrypted)');
            $table->string('url', 255)->nullable()->comment('Gateway base url');
            $table->string('submit_path', 255)->nullable()->comment('Submit path');
            $table->boolean('use_post')->default(true)->comment('Use POST submit');
            $table->string('sitename', 128)->nullable()->comment('Optional site name');
            $table->string('device', 128)->nullable()->comment('Optional device tag');
            $table->timestamps();

            $table->foreign('user_id')->references('id')->on('v2_user')->onDelete('cascade');
            $table->unique(['user_id', 'provider'], 'uniq_user_provider');
            $table->index(['provider']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('user_payment_profiles');
    }
};
