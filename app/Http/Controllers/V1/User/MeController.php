<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\User;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class MeController extends Controller
{
    public function show(Request $request): JsonResponse
    {
        /** @var User $user */
        $user = $request->user();

        $user = User::query()->whereKey($user->id)->first();
        if (!$user) {
            return $this->fail([400, __('The user does not exist')]);
        }

        return $this->success([
            'id' => $user->id,
            'email' => $user->email,
            'is_admin' => (bool) $user->is_admin,
            'is_super_admin' => (bool) $user->is_super_admin,
            'trust_level' => (int) ($user->trust_level ?? 0),
            'is_silenced' => (bool) ($user->is_silenced ?? false),
            'is_linux_do_user' => (bool) $user->isLinuxDoUser(),
            'linux_do_username' => $user->linux_do_username,
            'linux_do_name' => $user->linux_do_name,
            'linux_do_avatar' => $user->linux_do_avatar,
            'api_key' => $user->api_key,
            'concurrent_ip_limit' => (int) (($user->concurrent_ip_limit ?? 0) > 0 ? $user->concurrent_ip_limit : 3),
            'refund_dispute_enable' => (bool) admin_setting('refund_dispute_enable', 0),
            'secure_path' => $user->is_admin
                ? (string) admin_setting('secure_path', admin_setting('frontend_admin_path', hash('crc32b', config('app.key'))))
                : null,
        ]);
    }
}
