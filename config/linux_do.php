<?php

return [

    /*
    |--------------------------------------------------------------------------
    | Linux DO Connect OAuth2 Configuration
    |--------------------------------------------------------------------------
    |
    | This file contains the configuration for Linux DO Connect OAuth2
    | integration. You can obtain these credentials by registering your
    | application at https://connect.linux.do/
    |
    */

    'oauth' => [
        'client_id' => env('LINUX_DO_CLIENT_ID'),
        'client_secret' => env('LINUX_DO_CLIENT_SECRET'),
        'redirect_uri' => env('LINUX_DO_REDIRECT_URI', env('APP_URL') . '/api/v1/passport/oauth2/linux-do/callback'),
        'scopes' => ['read'],
    ],

    /*
    |--------------------------------------------------------------------------
    | API Endpoints
    |--------------------------------------------------------------------------
    |
    | Linux DO Connect API endpoints
    |
    */

    'endpoints' => [
        'authorize' => 'https://connect.linux.do/oauth2/authorize',
        'token' => 'https://connect.linux.do/oauth2/token',
        'user' => 'https://connect.linux.do/api/user',
    ],

    /*
    |--------------------------------------------------------------------------
    | User Sync Configuration
    |--------------------------------------------------------------------------
    |
    | Configuration for user information synchronization
    |
    */

    'sync' => [
        'auto_sync_on_login' => env('LINUX_DO_AUTO_SYNC', true),
        'sync_interval_hours' => env('LINUX_DO_SYNC_INTERVAL', 24),
        'auto_generate_api_key' => env('LINUX_DO_AUTO_API_KEY', true),
    ],

    /*
    |--------------------------------------------------------------------------
    | Trust Level Configuration
    |--------------------------------------------------------------------------
    |
    | Configuration for trust level based user grouping
    |
    */

    'trust_levels' => [
        0 => [
            'name' => 'New User',
            'description' => 'Newly registered users',
        ],
        1 => [
            'name' => 'Basic User',
            'description' => 'Users with basic trust level',
        ],
        2 => [
            'name' => 'Member',
            'description' => 'Regular community members',
        ],
        3 => [
            'name' => 'Regular',
            'description' => 'Trusted community members',
        ],
        4 => [
            'name' => 'Leader',
            'description' => 'Community leaders and moderators',
        ],
    ],

    /*
    |--------------------------------------------------------------------------
    | Default Limits Configuration
    |--------------------------------------------------------------------------
    |
    | Default limits for different trust levels
    |
    */

    'default_limits' => [
        0 => [
            'speed_limit_up' => env('LINUX_DO_LIMIT_0_UP', 50),
            'speed_limit_down' => env('LINUX_DO_LIMIT_0_DOWN', 100),
            'device_limit' => env('LINUX_DO_LIMIT_0_DEVICE', 2),
            'connection_limit' => env('LINUX_DO_LIMIT_0_CONNECTION', 5),
        ],
        1 => [
            'speed_limit_up' => env('LINUX_DO_LIMIT_1_UP', 100),
            'speed_limit_down' => env('LINUX_DO_LIMIT_1_DOWN', 200),
            'device_limit' => env('LINUX_DO_LIMIT_1_DEVICE', 3),
            'connection_limit' => env('LINUX_DO_LIMIT_1_CONNECTION', 10),
        ],
        2 => [
            'speed_limit_up' => env('LINUX_DO_LIMIT_2_UP', 200),
            'speed_limit_down' => env('LINUX_DO_LIMIT_2_DOWN', 500),
            'device_limit' => env('LINUX_DO_LIMIT_2_DEVICE', 5),
            'connection_limit' => env('LINUX_DO_LIMIT_2_CONNECTION', 20),
        ],
        3 => [
            'speed_limit_up' => env('LINUX_DO_LIMIT_3_UP', 500),
            'speed_limit_down' => env('LINUX_DO_LIMIT_3_DOWN', 1000),
            'device_limit' => env('LINUX_DO_LIMIT_3_DEVICE', 8),
            'connection_limit' => env('LINUX_DO_LIMIT_3_CONNECTION', 50),
        ],
        4 => [
            'speed_limit_up' => env('LINUX_DO_LIMIT_4_UP', 1000),
            'speed_limit_down' => env('LINUX_DO_LIMIT_4_DOWN', 2000),
            'device_limit' => env('LINUX_DO_LIMIT_4_DEVICE', 15),
            'connection_limit' => env('LINUX_DO_LIMIT_4_CONNECTION', 100),
        ],
    ],

    /*
    |--------------------------------------------------------------------------
    | API Key Configuration
    |--------------------------------------------------------------------------
    |
    | Configuration for API key generation and management
    |
    */

    'api_key' => [
        'prefix' => env('LINUX_DO_API_KEY_PREFIX', 'xb_'),
        'length' => env('LINUX_DO_API_KEY_LENGTH', 60),
        'auto_generate' => env('LINUX_DO_AUTO_GENERATE_API_KEY', true),
    ],

    /*
    |--------------------------------------------------------------------------
    | Security Configuration
    |--------------------------------------------------------------------------
    |
    | Security related configuration
    |
    */

    'security' => [
        'state_length' => 32,
        'token_encryption' => env('LINUX_DO_TOKEN_ENCRYPTION', true),
        'log_oauth_events' => env('LINUX_DO_LOG_OAUTH', true),
    ],

];