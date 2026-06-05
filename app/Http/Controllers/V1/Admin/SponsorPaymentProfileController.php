<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Support\UrlSecurity;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use InvalidArgumentException;

class SponsorPaymentProfileController extends Controller
{
    public function show(Request $request): JsonResponse
    {
        return response()->json([
            'success' => true,
            'data' => [
                'url' => (string) admin_setting('sponsor_epay_url', ''),
                'pid' => (string) admin_setting('sponsor_epay_pid', ''),
                'submit_path' => (string) admin_setting('sponsor_epay_submit_path', '/pay/submit.php'),
                'use_post' => (bool) admin_setting('sponsor_epay_use_post', 1),
                'sitename' => (string) admin_setting('sponsor_epay_sitename', ''),
                'device' => (string) admin_setting('sponsor_epay_device', ''),
                'has_key' => admin_setting('sponsor_epay_key', '') !== '',
            ],
        ]);
    }

    public function upsert(Request $request): JsonResponse
    {
        $data = $request->validate([
            'url' => 'required|string|max:255',
            'pid' => 'required|string|max:64',
            'key' => 'required|string|max:255',
            'submit_path' => 'nullable|string|max:255',
            'use_post' => 'nullable|boolean',
            'sitename' => 'nullable|string|max:128',
            'device' => 'nullable|string|max:128',
        ]);

        try {
            $url = UrlSecurity::normalizeHttpUrl($data['url']);
            $submitPath = UrlSecurity::normalizeRelativePath($data['submit_path'] ?? null, '/pay/submit.php');
        } catch (InvalidArgumentException $e) {
            return response()->json([
                'success' => false,
                'error' => $e->getMessage(),
            ], 422);
        }

        admin_setting([
            'sponsor_epay_url' => $url,
            'sponsor_epay_pid' => $data['pid'],
            'sponsor_epay_key' => $data['key'],
            'sponsor_epay_submit_path' => $submitPath,
            'sponsor_epay_use_post' => (int) ($data['use_post'] ?? true),
            'sponsor_epay_sitename' => $data['sitename'] ?? '',
            'sponsor_epay_device' => $data['device'] ?? '',
        ]);

        return response()->json(['success' => true]);
    }
}
