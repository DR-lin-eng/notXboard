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
        Schema::table('v2_user', function (Blueprint $table) {
            // Linux DO Connect OAuth2 相关字段
            $table->string('linux_do_id')->nullable()->unique()->after('uuid');
            $table->string('linux_do_username')->nullable()->after('linux_do_id');
            $table->string('linux_do_name')->nullable()->after('linux_do_username');
            $table->text('linux_do_avatar')->nullable()->after('linux_do_name');
            $table->tinyInteger('trust_level')->default(0)->after('linux_do_avatar');
            $table->boolean('is_silenced')->default(false)->after('trust_level');
            $table->json('external_ids')->nullable()->after('is_silenced');
            $table->string('api_key', 64)->nullable()->unique()->after('external_ids');
            $table->boolean('is_super_admin')->default(false)->after('is_admin');
            
            // OAuth2 令牌相关字段
            $table->string('oauth_provider', 50)->nullable()->after('api_key');
            $table->text('oauth_access_token')->nullable()->after('oauth_provider');
            $table->text('oauth_refresh_token')->nullable()->after('oauth_access_token');
            $table->timestamp('oauth_expires_at')->nullable()->after('oauth_refresh_token');
            
            // 添加索引
            $table->index('linux_do_id');
            $table->index('trust_level');
            $table->index('api_key');
            $table->index('is_super_admin');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('v2_user', function (Blueprint $table) {
            $table->dropIndex(['linux_do_id']);
            $table->dropIndex(['trust_level']);
            $table->dropIndex(['api_key']);
            $table->dropIndex(['is_super_admin']);
            
            $table->dropColumn([
                'linux_do_id',
                'linux_do_username',
                'linux_do_name',
                'linux_do_avatar',
                'trust_level',
                'is_silenced',
                'external_ids',
                'api_key',
                'is_super_admin',
                'oauth_provider',
                'oauth_access_token',
                'oauth_refresh_token',
                'oauth_expires_at'
            ]);
        });
    }
};