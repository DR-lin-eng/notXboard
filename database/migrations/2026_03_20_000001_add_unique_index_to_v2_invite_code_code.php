<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;
use Illuminate\Support\Str;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('v2_invite_code') || !Schema::hasColumn('v2_invite_code', 'code')) {
            return;
        }

        $this->deduplicateInviteCodes();

        try {
            Schema::table('v2_invite_code', function (Blueprint $table) {
                $table->unique('code', 'uniq_v2_invite_code_code');
            });
        } catch (\Throwable $e) {
            // Index already exists or cannot be created in current environment.
        }
    }

    public function down(): void
    {
        if (!Schema::hasTable('v2_invite_code')) {
            return;
        }

        try {
            Schema::table('v2_invite_code', function (Blueprint $table) {
                $table->dropUnique('uniq_v2_invite_code_code');
            });
        } catch (\Throwable $e) {
            // Ignore when index does not exist.
        }
    }

    private function deduplicateInviteCodes(): void
    {
        $duplicateCodes = DB::table('v2_invite_code')
            ->select('code')
            ->groupBy('code')
            ->havingRaw('COUNT(*) > 1')
            ->pluck('code');

        foreach ($duplicateCodes as $code) {
            $ids = DB::table('v2_invite_code')
                ->where('code', $code)
                ->orderBy('id')
                ->pluck('id')
                ->all();

            foreach (array_slice($ids, 1) as $id) {
                DB::table('v2_invite_code')
                    ->where('id', $id)
                    ->update(['code' => $this->generateUniqueInviteCode()]);
            }
        }
    }

    private function generateUniqueInviteCode(int $length = 8): string
    {
        do {
            $code = strtoupper(Str::random($length));
            $exists = DB::table('v2_invite_code')->where('code', $code)->exists();
        } while ($exists);

        return $code;
    }
};
