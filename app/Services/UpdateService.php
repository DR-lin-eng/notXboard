<?php

namespace App\Services;

class UpdateService
{
    public function getCurrentVersion(): string
    {
        return (string) config('app.version', '1.0.0');
    }

    public function updateVersionCache(): void
    {
        // Updates are disabled; no version cache refresh required.
    }

    public function checkForUpdates(): array
    {
        $current = $this->getCurrentVersion();

        return [
            'has_update' => false,
            'is_local_newer' => false,
            'latest_version' => $current,
            'current_version' => $current,
            'update_logs' => [],
            'download_url' => '',
            'published_at' => '',
            'author' => '',
        ];
    }

    public function executeUpdate(): array
    {
        return [
            'success' => false,
            'message' => 'Updates are disabled.'
        ];
    }
}
