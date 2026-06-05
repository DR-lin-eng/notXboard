<?php

namespace Tests\Unit;

use App\Http\Middleware\InitializePlugins;
use App\Services\Plugin\PluginManager;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Schema;
use Tests\TestCase;

class InitializePluginsMiddlewareTest extends TestCase
{
    public function test_middleware_allows_request_when_plugins_table_missing(): void
    {
        Schema::shouldReceive('hasTable')
            ->once()
            ->andReturn(false);

        $middleware = new InitializePlugins(new PluginManager());
        $request = Request::create('/status', 'GET');

        $response = $middleware->handle($request, function () {
            return response('ok', 200);
        });

        $this->assertSame(200, $response->getStatusCode());
        $this->assertSame('ok', $response->getContent());
    }
}
