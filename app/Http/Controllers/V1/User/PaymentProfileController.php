<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\UserPaymentProfile;
use App\Services\PaymentProfileService;
use App\Support\UrlSecurity;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Crypt;
use InvalidArgumentException;

class PaymentProfileController extends Controller
{
    public function showEpay(Request $request): JsonResponse
    {
        $user = Auth::user();

        $profile = UserPaymentProfile::query()
            ->where('user_id', $user->id)
            ->where('provider', PaymentProfileService::PROVIDER_EPAY)
            ->first();

        return response()->json([
            'success' => true,
            'data' => [
                'pid' => $profile?->pid,
                'url' => $profile?->url,
                'submit_path' => $profile?->submit_path,
                'use_post' => (bool) ($profile?->use_post ?? true),
                'sitename' => $profile?->sitename,
                'device' => $profile?->device,
                'has_key' => !empty($profile?->key_encrypted),
            ],
        ]);
    }

    public function upsertEpay(Request $request): JsonResponse
    {
        $user = Auth::user();

        $data = $request->validate([
            'pid' => 'required|string|max:64',
            'key' => 'required|string|max:255',
            'url' => 'required|string|max:255',
            'submit_path' => 'nullable|string|max:255',
            'use_post' => 'nullable|boolean',
            'sitename' => 'nullable|string|max:128',
            'device' => 'nullable|string|max:128',
        ]);

        try {
            $url = UrlSecurity::normalizeHttpUrl($data['url'], true);
            $submitPath = UrlSecurity::normalizeRelativePath($data['submit_path'] ?? null, '/pay/submit.php');
        } catch (InvalidArgumentException $e) {
            return response()->json([
                'success' => false,
                'error' => $e->getMessage(),
            ], 422);
        }

        $profile = UserPaymentProfile::query()->updateOrCreate(
            ['user_id' => $user->id, 'provider' => PaymentProfileService::PROVIDER_EPAY],
            [
                'pid' => $data['pid'],
                'key_encrypted' => Crypt::encryptString($data['key']),
                'url' => $url,
                'submit_path' => $submitPath,
                'use_post' => $data['use_post'] ?? true,
                'sitename' => $data['sitename'] ?? null,
                'device' => $data['device'] ?? null,
            ]
        );

        return response()->json([
            'success' => true,
            'data' => [
                'pid' => $profile->pid,
                'url' => $profile->url,
                'submit_path' => $profile->submit_path,
                'use_post' => (bool) $profile->use_post,
                'sitename' => $profile->sitename,
                'device' => $profile->device,
                'has_key' => true,
            ],
        ]);
    }
}
