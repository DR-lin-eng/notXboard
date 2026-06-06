<?php

use Illuminate\Support\Facades\Route;
use Illuminate\Support\Facades\Schema;

/*
|--------------------------------------------------------------------------
| Web Routes
|--------------------------------------------------------------------------
|
| The supported default HTTP surface is owned by rust-gateway. Laravel keeps
| only a minimal maintenance health check route here.
|
*/

Route::get('/healthz', function () {
    try {
        if (!Schema::hasTable('migrations') || !Schema::hasTable('v2_settings')) {
            return response('not ready', 503, ['Cache-Control' => 'no-store']);
        }
    } catch (\Throwable) {
        return response('not ready', 503, ['Cache-Control' => 'no-store']);
    }

    return response('ok', 200, ['Cache-Control' => 'no-store']);
})->name('healthz');
