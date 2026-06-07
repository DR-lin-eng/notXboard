<?php

namespace App\Services;

use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\View;
use Illuminate\Http\UploadedFile;
use Exception;
use ZipArchive;

class ThemeService
{
    public const DEFAULT_THEME = 'Maintainable';
    private const SYSTEM_THEME_DIR = 'theme/';
    private const USER_THEME_DIR = '/storage/theme/';
    private const CONFIG_FILE = 'config.json';
    private const SETTING_PREFIX = 'theme_';
    private const SYSTEM_THEMES = [self::DEFAULT_THEME, 'portal'];
    private const THEME_NAME_PATTERN = '/^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/';
    private const MAX_ARCHIVE_BYTES = 10485760;
    private const MAX_ARCHIVE_ENTRIES = 512;
    private const MAX_EXTRACTED_BYTES = 52428800;
    private const MAX_ENTRY_BYTES = 10485760;

    /**
     * Normalize theme name and fallback to system default when empty.
     */
    private function normalizeThemeName(?string $theme): string
    {
        $theme = trim((string) $theme);
        return $theme !== '' ? $theme : self::DEFAULT_THEME;
    }

    private function isValidThemeName(string $theme): bool
    {
        return preg_match(self::THEME_NAME_PATTERN, $theme) === 1
            && !str_contains($theme, '..')
            && !str_contains($theme, '/')
            && !str_contains($theme, '\\');
    }

    private function assertValidThemeName(string $theme): string
    {
        $normalized = $this->normalizeThemeName($theme);
        if (!$this->isValidThemeName($normalized)) {
            throw new Exception('Invalid theme name');
        }

        return $normalized;
    }

    private function extractZipSafely(ZipArchive $zip, string $destination): void
    {
        File::ensureDirectoryExists($destination);
        if ($zip->numFiles > self::MAX_ARCHIVE_ENTRIES) {
            throw new Exception('Theme package contains too many files');
        }

        $totalBytes = 0;
        for ($i = 0; $i < $zip->numFiles; $i++) {
            $entryName = (string) $zip->getNameIndex($i);
            if ($entryName === '') {
                continue;
            }

            $normalized = str_replace('\\', '/', $entryName);
            $normalized = ltrim($normalized, '/');

            if ($normalized === '' || preg_match('#(^|/)\.\.(?:/|$)#', $normalized)) {
                throw new Exception('Theme package contains unsafe paths');
            }

            $targetPath = $destination . '/' . $normalized;

            if (str_ends_with($normalized, '/')) {
                File::ensureDirectoryExists($targetPath);
                continue;
            }

            $stat = $zip->statIndex($i);
            if ($stat === false) {
                throw new Exception('Failed to read theme package entry metadata');
            }
            $declaredSize = (int) ($stat['size'] ?? 0);
            if ($declaredSize > self::MAX_ENTRY_BYTES || $totalBytes + $declaredSize > self::MAX_EXTRACTED_BYTES) {
                throw new Exception('Theme package is too large');
            }

            File::ensureDirectoryExists(dirname($targetPath));

            $stream = $zip->getStream($entryName);
            if ($stream === false) {
                throw new Exception('Failed to read theme package entry');
            }

            $output = fopen($targetPath, 'wb');
            if ($output === false) {
                fclose($stream);
                throw new Exception('Failed to create theme package entry');
            }

            $entryBytes = 0;
            try {
                while (!feof($stream)) {
                    $chunk = fread($stream, 8192);
                    if ($chunk === false) {
                        throw new Exception('Failed to extract theme package entry');
                    }
                    $entryBytes += strlen($chunk);
                    $totalBytes += strlen($chunk);
                    if ($entryBytes > self::MAX_ENTRY_BYTES || $totalBytes > self::MAX_EXTRACTED_BYTES) {
                        throw new Exception('Theme package is too large');
                    }
                    if (fwrite($output, $chunk) === false) {
                        throw new Exception('Failed to extract theme package entry');
                    }
                }
            } finally {
                fclose($stream);
                fclose($output);
            }
        }
    }

    public function __construct()
    {
        $this->registerThemeViewPaths();
    }

    /**
     * Register theme view paths
     */
    private function registerThemeViewPaths(): void
    {
        $systemPath = base_path(self::SYSTEM_THEME_DIR);
        if (File::exists($systemPath)) {
            View::addNamespace('theme', $systemPath);
        }

        $userPath = base_path(self::USER_THEME_DIR);
        if (File::exists($userPath)) {
            View::prependNamespace('theme', $userPath);
        }
    }

    /**
     * Get theme view path
     */
    public function getThemeViewPath(string $theme): ?string
    {
        $themePath = $this->getThemePath($theme);
        if (!$themePath) {
            return null;
        }
        return $themePath . '/dashboard.html';
    }

    /**
     * Get all available themes
     */
    public function getList(): array
    {
        $themes = [];

        // 获取系统主题
        $systemPath = base_path(self::SYSTEM_THEME_DIR);
        if (File::exists($systemPath)) {
            $themes = $this->getThemesFromPath($systemPath, false);
        }

        // 获取用户主题
        $userPath = base_path(self::USER_THEME_DIR);
        if (File::exists($userPath)) {
            $themes = array_merge($themes, $this->getThemesFromPath($userPath, true));
        }

        return $themes;
    }

    /**
     * Get themes from specified path
     */
    private function getThemesFromPath(string $path, bool $canDelete): array
    {
        return collect(File::directories($path))
            ->mapWithKeys(function ($dir) use ($canDelete) {
                $name = basename($dir);
                if (
                    !File::exists($dir . '/' . self::CONFIG_FILE) ||
                    !File::exists($dir . '/dashboard.html')
                ) {
                    return [];
                }
                $config = $this->readConfigFile($name);
                if (!$config) {
                    return [];
                }

                $config['can_delete'] = $canDelete && $name !== admin_setting('current_theme');
                $config['is_system'] = !$canDelete;
                return [$name => $config];
            })->toArray();
    }

    /**
     * Upload new theme
     */
    public function upload(UploadedFile $file): bool
    {
        if (($file->getSize() ?: 0) > self::MAX_ARCHIVE_BYTES) {
            throw new Exception('Theme package size cannot exceed 10MB');
        }

        $zip = new ZipArchive;
        $tmpPath = storage_path('tmp/' . uniqid());

        try {
            if ($zip->open($file->path()) !== true) {
                throw new Exception('Invalid theme package');
            }

            $configEntry = collect(range(0, $zip->numFiles - 1))
                ->map(fn($i) => $zip->getNameIndex($i))
                ->first(fn($name) => basename($name) === self::CONFIG_FILE);

            if (!$configEntry) {
                throw new Exception('Theme config file not found');
            }

            $this->extractZipSafely($zip, $tmpPath);
            $zip->close();

            $sourcePath = $tmpPath . '/' . rtrim(dirname($configEntry), '.');
            $configFile = $sourcePath . '/' . self::CONFIG_FILE;

            if (!File::exists($configFile)) {
                throw new Exception('Theme config file not found');
            }

            $config = json_decode(File::get($configFile), true);
            if (empty($config['name'])) {
                throw new Exception('Theme name not configured');
            }

            $themeName = $this->assertValidThemeName((string) $config['name']);

            if (in_array($themeName, self::SYSTEM_THEMES, true)) {
                throw new Exception('Cannot upload theme with same name as system theme');
            }

            if (!File::exists($sourcePath . '/dashboard.html')) {
                throw new Exception('Missing required theme file: dashboard.html');
            }

            $userThemePath = base_path(self::USER_THEME_DIR);
            if (!File::exists($userThemePath)) {
                File::makeDirectory($userThemePath, 0755, true);
            }

            $targetPath = $userThemePath . $themeName;
            if (File::exists($targetPath)) {
                $oldConfigFile = $targetPath . '/config.json';
                if (!File::exists($oldConfigFile)) {
                    throw new Exception('Existing theme missing config file');
                }
                $oldConfig = json_decode(File::get($oldConfigFile), true);
                $oldVersion = $oldConfig['version'] ?? '0.0.0';
                $newVersion = $config['version'] ?? '0.0.0';
                if (version_compare($newVersion, $oldVersion, '>')) {
                    $this->cleanupThemeFiles($themeName);
                    File::deleteDirectory($targetPath);
                    File::copyDirectory($sourcePath, $targetPath);
                    // 更新主题时保留用户配置
                    $this->initConfig($themeName, true);
                    return true;
                } else {
                    throw new Exception('Theme exists and not a newer version');
                }
            }

            File::copyDirectory($sourcePath, $targetPath);
            $this->initConfig($themeName);

            return true;

        } catch (Exception $e) {
            throw $e;
        } finally {
            if (File::exists($tmpPath)) {
                File::deleteDirectory($tmpPath);
            }
        }
    }

    /**
     * Switch theme
     */
    public function switch(string|null $theme): bool
    {
        if ($theme === null) {
            return true;
        }

        $theme = $this->assertValidThemeName($theme);
        $currentTheme = $this->normalizeThemeName(admin_setting('current_theme'));

        try {
            $themePath = $this->getThemePath($theme);
            if (!$themePath) {
                throw new Exception('Theme not found');
            }

            if (!File::exists($this->getThemeViewPath($theme))) {
                throw new Exception('Theme view file not found');
            }

            if ($currentTheme && $currentTheme !== $theme) {
                $this->cleanupThemeFiles($currentTheme);
            }

            $targetPath = public_path('theme/' . $theme);
            if (File::exists($targetPath)) {
                File::deleteDirectory($targetPath);
            }

            if (!File::copyDirectory($themePath, $targetPath)) {
                throw new Exception('Failed to copy theme files');
            }

            admin_setting([
                'current_theme' => $theme,
                'frontend_theme' => $theme
            ]);
            return true;

        } catch (Exception $e) {
            Log::error('Theme switch failed', ['theme' => $theme, 'error' => $e->getMessage()]);
            throw $e;
        }
    }

    /**
     * Delete theme
     */
    public function delete(string $theme): bool
    {
        try {
            $theme = $this->assertValidThemeName($theme);
            if (in_array($theme, self::SYSTEM_THEMES)) {
                throw new Exception('System theme cannot be deleted');
            }

            if ($theme === admin_setting('current_theme')) {
                throw new Exception('Current theme cannot be deleted');
            }

            $themePath = base_path(self::USER_THEME_DIR . $theme);
            if (!File::exists($themePath)) {
                throw new Exception('Theme not found');
            }

            $this->cleanupThemeFiles($theme);
            File::deleteDirectory($themePath);
            admin_setting([self::SETTING_PREFIX . $theme => null]);
            return true;

        } catch (Exception $e) {
            Log::error('Theme deletion failed', ['theme' => $theme, 'error' => $e->getMessage()]);
            throw $e;
        }
    }

    /**
     * Check if theme exists
     */
    public function exists(string $theme): bool
    {
        return $this->getThemePath($theme) !== null;
    }

    /**
     * Get theme path
     */
    public function getThemePath(string $theme): ?string
    {
        $theme = $this->normalizeThemeName($theme);
        if (!$this->isValidThemeName($theme)) {
            return null;
        }
        $systemPath = base_path(self::SYSTEM_THEME_DIR . $theme);
        if (File::exists($systemPath)) {
            return $systemPath;
        }

        $userPath = base_path(self::USER_THEME_DIR . $theme);
        if (File::exists($userPath)) {
            return $userPath;
        }

        return null;
    }

    /**
     * Get theme config
     */
    public function getConfig(string $theme): ?array
    {
        $theme = $this->assertValidThemeName($theme);
        $config = admin_setting(self::SETTING_PREFIX . $theme);
        if ($config === null) {
            $this->initConfig($theme);
            $config = admin_setting(self::SETTING_PREFIX . $theme);
        }
        return $config;
    }

    /**
     * Update theme config
     */
    public function updateConfig(string $theme, array $config): bool
    {
        $theme = $this->assertValidThemeName($theme);
        try {
            if (!$this->getThemePath($theme)) {
                throw new Exception('Theme not found');
            }

            $schema = $this->readConfigFile($theme);
            if (!$schema) {
                throw new Exception('Invalid theme config file');
            }

            $validFields = collect($schema['configs'] ?? [])->pluck('field_name')->toArray();
            $validConfig = collect($config)
                ->only($validFields)
                ->toArray();

            $currentConfig = $this->getConfig($theme) ?? [];
            $newConfig = array_merge($currentConfig, $validConfig);

            admin_setting([self::SETTING_PREFIX . $theme => $newConfig]);
            return true;

        } catch (Exception $e) {
            Log::error('Config update failed', ['theme' => $theme, 'error' => $e->getMessage()]);
            throw $e;
        }
    }

    /**
     * Read theme config file
     */
    private function readConfigFile(string $theme): ?array
    {
        $themePath = $this->getThemePath($theme);
        if (!$themePath) {
            return null;
        }

        $file = $themePath . '/' . self::CONFIG_FILE;
        return File::exists($file) ? json_decode(File::get($file), true) : null;
    }

    /**
     * Clean up theme files including public directory
     */
    public function cleanupThemeFiles(string $theme): void
    {
        try {
            $theme = $this->assertValidThemeName($theme);
            $publicThemePath = public_path('theme/' . $theme);
            if (File::exists($publicThemePath)) {
                File::deleteDirectory($publicThemePath);
                Log::info('Cleaned up public theme files', ['theme' => $theme, 'path' => $publicThemePath]);
            }

            $cacheKey = "theme_{$theme}_assets";
            if (cache()->has($cacheKey)) {
                cache()->forget($cacheKey);
                Log::info('Cleaned up theme cache', ['theme' => $theme, 'cache_key' => $cacheKey]);
            }

        } catch (Exception $e) {
            Log::warning('Failed to cleanup theme files', [
                'theme' => $theme,
                'error' => $e->getMessage()
            ]);
        }
    }

    /**
     * Force refresh current theme public files
     */
    public function refreshCurrentTheme(): bool
    {
        try {
            $currentTheme = $this->assertValidThemeName((string) admin_setting('current_theme'));
            if (!$currentTheme) {
                return false;
            }

            $this->cleanupThemeFiles($currentTheme);

            $themePath = $this->getThemePath($currentTheme);
            if (!$themePath) {
                throw new Exception('Current theme path not found');
            }

            $targetPath = public_path('theme/' . $currentTheme);
            if (!File::copyDirectory($themePath, $targetPath)) {
                throw new Exception('Failed to copy theme files');
            }

            Log::info('Refreshed current theme files', ['theme' => $currentTheme]);
            return true;

        } catch (Exception $e) {
            Log::error('Failed to refresh current theme', [
                'theme' => $currentTheme,
                'error' => $e->getMessage()
            ]);
            return false;
        }
    }

    /**
     * Initialize theme config
     * 
     * @param string $theme 主题名称
     * @param bool $preserveExisting 是否保留现有配置（更新主题时使用）
     */
    private function initConfig(string $theme, bool $preserveExisting = false): void
    {
        $theme = $this->assertValidThemeName($theme);
        $config = $this->readConfigFile($theme);
        if (!$config) {
            return;
        }

        $defaults = collect($config['configs'] ?? [])
            ->mapWithKeys(fn($col) => [$col['field_name'] => $col['default_value'] ?? ''])
            ->toArray();

        if ($preserveExisting) {
            $existingConfig = admin_setting(self::SETTING_PREFIX . $theme) ?? [];
            $mergedConfig = array_merge($defaults, $existingConfig);
            admin_setting([self::SETTING_PREFIX . $theme => $mergedConfig]);
        } else {
            admin_setting([self::SETTING_PREFIX . $theme => $defaults]);
        }
    }
}
