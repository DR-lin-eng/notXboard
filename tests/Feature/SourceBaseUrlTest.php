<?php

namespace Tests\Feature;

use Illuminate\Http\Request;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class SourceBaseUrlTest extends TestCase
{
    use RefreshDatabase;

    public function test_uses_request_host_when_configured_app_url_is_loopback(): void
    {
        config(['app.url' => 'http://127.0.0.1:8000']);
        $this->app->instance('request', Request::create('https://billing.example.com/account', 'GET'));

        $this->assertSame('https://billing.example.com/callback', source_base_url('/callback'));
    }

    public function test_keeps_configured_public_app_url_when_it_is_not_local(): void
    {
        config(['app.url' => 'https://portal.example.com']);
        $this->app->instance('request', Request::create('https://billing.example.com/account', 'GET'));

        $this->assertSame('https://portal.example.com/callback', source_base_url('/callback'));
    }
}
