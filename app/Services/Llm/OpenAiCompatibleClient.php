<?php

namespace App\Services\Llm;

use App\Support\UrlSecurity;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

class OpenAiCompatibleClient
{
    private const ENDPOINT_CHAT_COMPLETIONS = 'chat_completions';
    private const ENDPOINT_RESPONSES = 'responses';

    public function isConfigured(): bool
    {
        return (bool) admin_setting('user_risk_review_llm_enable', 0)
            && trim((string) admin_setting('user_risk_review_llm_base_url', '')) !== ''
            && trim((string) admin_setting('user_risk_review_llm_api_key', '')) !== ''
            && trim((string) admin_setting('user_risk_review_llm_model', '')) !== '';
    }

    public function getModel(): string
    {
        return trim((string) admin_setting('user_risk_review_llm_model', ''));
    }

    public function reviewRisk(string $systemPrompt, string $userPrompt): array
    {
        if (!$this->isConfigured()) {
            throw new \RuntimeException('LLM 风控审查未启用或配置不完整');
        }

        $endpoint = $this->buildEndpoint();
        $model = $this->getModel();
        $timeout = max(5, (int) admin_setting('user_risk_review_llm_timeout_seconds', 20));
        $temperature = max(0, min(1, (float) admin_setting('user_risk_review_llm_temperature', 0.2)));

        $response = Http::timeout($timeout)
            ->connectTimeout(min(5, $timeout))
            ->retry(2, 1000)
            ->acceptJson()
            ->asJson()
            ->withToken((string) admin_setting('user_risk_review_llm_api_key', ''))
            ->post($endpoint['url'], $this->buildRequestPayload(
                $endpoint['type'],
                $model,
                $temperature,
                $systemPrompt,
                $userPrompt
            ));

        if (!$response->successful()) {
            $errorMessage = $response->json('error.message')
                ?: $response->json('message')
                ?: Str::limit(trim((string) $response->body()), 240, '...');

            throw new \RuntimeException(
                'LLM 审查请求失败: HTTP ' . $response->status() . ($errorMessage !== '' ? ' - ' . $errorMessage : '')
            );
        }

        $payload = $response->json();
        $content = $this->extractTextContent($payload);
        $parsed = $this->extractStructuredPayload($content);

        Log::info('User risk review LLM completed', [
            'endpoint' => $endpoint['url'],
            'endpoint_type' => $endpoint['type'],
            'model' => $payload['model'] ?? $model,
        ]);

        return [
            'model' => (string) ($payload['model'] ?? $model),
            'content' => $content,
            'parsed' => $parsed,
            'raw' => $payload,
        ];
    }

    private function buildEndpoint(): array
    {
        $normalizedUrl = $this->normalizedBaseUrl();
        $path = (string) (parse_url($normalizedUrl, PHP_URL_PATH) ?: '');

        if (Str::endsWith($path, '/responses')) {
            return [
                'type' => self::ENDPOINT_RESPONSES,
                'url' => $normalizedUrl,
            ];
        }

        if (Str::endsWith($path, '/chat/completions')) {
            return [
                'type' => self::ENDPOINT_CHAT_COMPLETIONS,
                'url' => $normalizedUrl,
            ];
        }

        if ($normalizedUrl === '' || preg_match('#^https?://[^/]+$#', $normalizedUrl) === 1) {
            return [
                'type' => self::ENDPOINT_CHAT_COMPLETIONS,
                'url' => $this->appendPathToBaseUrl($normalizedUrl, '/v1/chat/completions'),
            ];
        }

        if (Str::endsWith($path, '/v1')) {
            return [
                'type' => self::ENDPOINT_CHAT_COMPLETIONS,
                'url' => $this->appendPathToBaseUrl($normalizedUrl, '/chat/completions'),
            ];
        }

        return [
            'type' => self::ENDPOINT_CHAT_COMPLETIONS,
            'url' => $this->appendPathToBaseUrl($normalizedUrl, '/chat/completions'),
        ];
    }

    private function buildRequestPayload(
        string $endpointType,
        string $model,
        float $temperature,
        string $systemPrompt,
        string $userPrompt
    ): array {
        if ($endpointType === self::ENDPOINT_RESPONSES) {
            return [
                'model' => $model,
                'temperature' => $temperature,
                'input' => [
                    ['role' => 'system', 'content' => $systemPrompt],
                    ['role' => 'user', 'content' => $userPrompt],
                ],
            ];
        }

        return [
            'model' => $model,
            'temperature' => $temperature,
            'messages' => [
                ['role' => 'system', 'content' => $systemPrompt],
                ['role' => 'user', 'content' => $userPrompt],
            ],
        ];
    }

    private function normalizedBaseUrl(): string
    {
        $baseUrl = trim((string) admin_setting('user_risk_review_llm_base_url', ''));
        try {
            $normalizedUrl = UrlSecurity::normalizeHttpUrl($baseUrl);
        } catch (\InvalidArgumentException $e) {
            throw new \RuntimeException('LLM Base URL 配置不合法: ' . $e->getMessage());
        }

        if (parse_url($normalizedUrl, PHP_URL_QUERY) !== null || parse_url($normalizedUrl, PHP_URL_FRAGMENT) !== null) {
            throw new \RuntimeException('LLM Base URL 不允许包含 query 或 fragment');
        }

        return $normalizedUrl;
    }

    private function appendPathToBaseUrl(string $baseUrl, string $suffix): string
    {
        $parts = parse_url($baseUrl);
        if ($parts === false) {
            return rtrim($baseUrl, '/') . $suffix;
        }

        $scheme = isset($parts['scheme']) ? $parts['scheme'] . '://' : '';
        $auth = '';
        if (isset($parts['user'])) {
            $auth = $parts['user'];
            if (isset($parts['pass'])) {
                $auth .= ':' . $parts['pass'];
            }
            $auth .= '@';
        }

        $host = $parts['host'] ?? '';
        $port = isset($parts['port']) ? ':' . $parts['port'] : '';
        $path = rtrim((string) ($parts['path'] ?? ''), '/');
        $query = isset($parts['query']) ? '?' . $parts['query'] : '';
        $fragment = isset($parts['fragment']) ? '#' . $parts['fragment'] : '';

        return $scheme . $auth . $host . $port . $path . $suffix . $query . $fragment;
    }

    private function extractTextContent(array $payload): string
    {
        $messageContent = data_get($payload, 'choices.0.message.content');
        if (is_string($messageContent)) {
            return trim($messageContent);
        }

        if (is_array($messageContent)) {
            $text = collect($messageContent)
                ->map(function ($item): string {
                    if (is_string($item)) {
                        return $item;
                    }

                    return (string) data_get($item, 'text', '');
                })
                ->filter()
                ->implode("\n");

            if ($text !== '') {
                return trim($text);
            }
        }

        $responseOutputText = $this->extractResponsesOutputText($payload);
        if ($responseOutputText !== '') {
            return $responseOutputText;
        }

        $outputText = data_get($payload, 'output_text');
        if (is_string($outputText) && trim($outputText) !== '') {
            return trim($outputText);
        }

        throw new \RuntimeException('LLM 返回内容为空');
    }

    private function extractResponsesOutputText(array $payload): string
    {
        $output = data_get($payload, 'output');
        if (!is_array($output)) {
            return '';
        }

        $parts = [];

        foreach ($output as $item) {
            if (is_string($item) && trim($item) !== '') {
                $parts[] = trim($item);
                continue;
            }

            if (!is_array($item)) {
                continue;
            }

            $contentItems = data_get($item, 'content');
            if (!is_array($contentItems)) {
                continue;
            }

            foreach ($contentItems as $contentItem) {
                if (is_string($contentItem) && trim($contentItem) !== '') {
                    $parts[] = trim($contentItem);
                    continue;
                }

                if (!is_array($contentItem)) {
                    continue;
                }

                $text = $contentItem['text'] ?? null;
                if (is_string($text) && trim($text) !== '') {
                    $parts[] = trim($text);
                }
            }
        }

        return trim(implode("\n", $parts));
    }

    private function extractStructuredPayload(string $content): array
    {
        $trimmed = trim($content);
        if ($trimmed === '') {
            throw new \RuntimeException('LLM 返回内容为空');
        }

        $candidates = [$trimmed];

        if (preg_match('/```(?:json)?\s*(\{.*\})\s*```/is', $trimmed, $matches) === 1) {
            $candidates[] = trim($matches[1]);
        }

        $start = strpos($trimmed, '{');
        $end = strrpos($trimmed, '}');
        if ($start !== false && $end !== false && $end > $start) {
            $candidates[] = substr($trimmed, $start, $end - $start + 1);
        }

        foreach ($candidates as $candidate) {
            $decoded = json_decode($candidate, true);
            if (json_last_error() === JSON_ERROR_NONE && is_array($decoded)) {
                return $decoded;
            }
        }

        throw new \RuntimeException('LLM 未返回可解析 JSON');
    }
}
