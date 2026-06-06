<?php

return [
    'telegram_only_mode' => filter_var(
        (string) env('TELEGRAM_ONLY_MODE', false),
        FILTER_VALIDATE_BOOLEAN
    ),
    'mail_sync_send' => true,
    'telegram_sync_send' => true,
    'core_job_sync_execution' => true,
];
