<?php

namespace Tests\Unit;

use App\Logging\MysqlLoggerHandler;
use Illuminate\Http\Request;
use Tests\TestCase;

class MysqlLoggerHandlerTest extends TestCase
{
    public function test_safe_request_payload_does_not_include_request_body_values(): void
    {
        $handler = new MysqlLoggerHandler();
        $request = Request::create('/ops/check?keyword=urgent&token=abc123', 'POST', [
            'password' => 'secret',
            'payload' => 'very-sensitive-body',
        ]);

        $method = new \ReflectionMethod($handler, 'buildSafeRequestData');
        $method->setAccessible(true);

        $safeData = $method->invoke($handler, $request);

        $this->assertSame('http', $safeData['type']);
        $this->assertContains('keyword', $safeData['query_keys']);
        $this->assertContains('token', $safeData['query_keys']);
        $this->assertStringNotContainsString('very-sensitive-body', json_encode($safeData));
        $this->assertStringNotContainsString('secret', json_encode($safeData));
    }

    public function test_sensitive_context_keys_are_redacted(): void
    {
        $handler = new MysqlLoggerHandler();
        $method = new \ReflectionMethod($handler, 'sanitizeValue');
        $method->setAccessible(true);

        $sanitized = $method->invoke($handler, [
            'password' => 'raw-password',
            'nested' => [
                'token' => 'raw-token',
                'safe_key' => 'visible-value',
            ],
        ]);

        $this->assertSame('[redacted]', $sanitized['password']);
        $this->assertSame('[redacted]', $sanitized['nested']['token']);
        $this->assertSame('visible-value', $sanitized['nested']['safe_key']);
    }
}
