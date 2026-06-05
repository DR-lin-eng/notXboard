<?php

return [
    'telegram_only_mode' => filter_var(
        (string) env('TELEGRAM_ONLY_MODE', false),
        FILTER_VALIDATE_BOOLEAN
    ),
    'mail_sync_send' => filter_var(
        (string) env('MAIL_SYNC_SEND', true),
        FILTER_VALIDATE_BOOLEAN
    ),
    'telegram_sync_send' => filter_var(
        (string) env('TELEGRAM_SYNC_SEND', true),
        FILTER_VALIDATE_BOOLEAN
    ),
    'core_job_sync_execution' => filter_var(
        (string) env('CORE_JOB_SYNC_EXECUTION', true),
        FILTER_VALIDATE_BOOLEAN
    ),
];
