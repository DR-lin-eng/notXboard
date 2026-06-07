<?php

return [
    'telegram_only_mode' => filter_var(
        (string) env('TELEGRAM_ONLY_MODE', false),
        FILTER_VALIDATE_BOOLEAN
    ),
];
