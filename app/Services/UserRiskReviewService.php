<?php

namespace App\Services;

use App\Models\AuditLog;
use App\Models\NodeTrafficRecord;
use App\Models\User;
use App\Models\UserOnlineSession;
use App\Models\UserRiskReview;
use App\Models\UserTrafficUsageLog;
use App\Services\Llm\OpenAiCompatibleClient;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Throwable;

class UserRiskReviewService
{
    private const LAST_RUN_CACHE_KEY = 'user_risk_review:last_run_at';

    public function __construct(
        private readonly OpenAiCompatibleClient $llmClient
    ) {
    }

    public function scanAndReview(?int $limit = null, bool $force = false): array
    {
        if (!$force && !(bool) admin_setting('user_risk_review_enable', 0)) {
            return [
                'skipped' => true,
                'reason' => '风险审查未启用',
            ];
        }

        if (!$force && !$this->shouldRunNow()) {
            return [
                'skipped' => true,
                'reason' => '未到下一次审查时间',
            ];
        }

        $groupLimit = $limit ?: max(1, (int) admin_setting('user_risk_review_scan_limit', 20));
        $sharedIpGroups = $this->findSharedIpGroups($groupLimit);

        $summary = [
            'skipped' => false,
            'groups_scanned' => $sharedIpGroups->count(),
            'reviews_created' => 0,
            'users_skipped' => 0,
            'telegram_notifications' => 0,
            'shared_ips' => [],
        ];

        foreach ($sharedIpGroups as $group) {
            $groupResult = $this->reviewSharedIpGroup((string) $group->ip_address, (int) $group->matched_user_count);
            $summary['reviews_created'] += (int) $groupResult['reviews_created'];
            $summary['users_skipped'] += (int) $groupResult['users_skipped'];
            $summary['telegram_notifications'] += (int) $groupResult['telegram_notifications'];
            $summary['shared_ips'][] = $groupResult;
        }

        Cache::put(self::LAST_RUN_CACHE_KEY, time(), now()->addDay());

        return $summary;
    }

    public function getRecentReviews(int $page = 1, int $pageSize = 20)
    {
        return UserRiskReview::query()
            ->with(['user:id,email,banned,ban_reason,plan_id'])
            ->orderByDesc('reviewed_at')
            ->paginate($pageSize, ['*'], 'page', $page);
    }

    private function shouldRunNow(): bool
    {
        $lastRunAt = (int) Cache::get(self::LAST_RUN_CACHE_KEY, 0);
        $intervalMinutes = max(5, (int) admin_setting('user_risk_review_schedule_minutes', 30));

        return $lastRunAt <= 0 || (time() - $lastRunAt) >= ($intervalMinutes * 60);
    }

    private function findSharedIpGroups(int $limit): Collection
    {
        $windowStart = now()->subMinutes(max(5, (int) admin_setting('user_risk_review_time_window_minutes', 60)));
        $minSharedUsers = max(2, (int) admin_setting('user_risk_review_min_shared_ip_users', 2));

        return UserOnlineSession::query()
            ->select('ip_address')
            ->selectRaw('COUNT(DISTINCT user_id) as matched_user_count')
            ->selectRaw('MAX(last_activity) as last_seen_at')
            ->where('last_activity', '>=', $windowStart)
            ->groupBy('ip_address')
            ->havingRaw('COUNT(DISTINCT user_id) >= ?', [$minSharedUsers])
            ->orderByDesc('matched_user_count')
            ->orderByDesc(DB::raw('MAX(last_activity)'))
            ->limit($limit)
            ->get();
    }

    private function reviewSharedIpGroup(string $sharedIp, int $matchedUserCount): array
    {
        $windowStart = now()->subMinutes(max(5, (int) admin_setting('user_risk_review_time_window_minutes', 60)));
        $cooldownMinutes = max(5, (int) admin_setting('user_risk_review_notify_cooldown_minutes', 60));

        $sessions = UserOnlineSession::query()
            ->with(['node:id,name,protocol,location_name'])
            ->where('ip_address', $sharedIp)
            ->where('last_activity', '>=', $windowStart)
            ->orderByDesc('last_activity')
            ->get();

        $userIds = $sessions->pluck('user_id')->unique()->values()->all();
        $users = User::query()
            ->with('plan:id,name')
            ->whereIn('id', $userIds)
            ->get()
            ->keyBy('id');

        $createdReviews = collect();
        $skippedUsers = 0;

        foreach ($userIds as $userId) {
            /** @var User|null $user */
            $user = $users->get($userId);
            if (!$user || $user->banned) {
                $skippedUsers++;
                continue;
            }

            if ($this->wasReviewedRecently((int) $user->id, $sharedIp, $cooldownMinutes)) {
                $skippedUsers++;
                continue;
            }

            $reviewContext = $this->buildReviewContext($user, $sharedIp, $matchedUserCount, $sessions, $users);
            $result = $this->reviewUserContext($reviewContext);

            $createdReviews->push(
                UserRiskReview::query()->create([
                    'user_id' => $user->id,
                    'source' => 'shared_ip',
                    'shared_ip' => $sharedIp,
                    'matched_user_count' => $matchedUserCount,
                    'matched_user_ids' => $reviewContext['matched_users'],
                    'risk_level' => $result['risk_level'],
                    'suspicion_score' => $result['suspicion_score'],
                    'llm_model' => $result['llm_model'],
                    'summary' => $result['summary'],
                    'recommendation' => $result['recommendation'],
                    'raw_response' => $result['raw_response'],
                    'evidence' => $reviewContext['evidence'],
                    'reviewed_at' => time(),
                    'created_at' => time(),
                    'updated_at' => time(),
                ])
            );
        }

        $notifications = 0;
        if ($createdReviews->isNotEmpty()) {
            $notifications = $this->sendTelegramAlert($sharedIp, $matchedUserCount, $createdReviews, $users);
        }

        return [
            'shared_ip' => $sharedIp,
            'matched_user_count' => $matchedUserCount,
            'reviews_created' => $createdReviews->count(),
            'users_skipped' => $skippedUsers,
            'telegram_notifications' => $notifications,
        ];
    }

    private function buildReviewContext(
        User $user,
        string $sharedIp,
        int $matchedUserCount,
        Collection $sessions,
        Collection $users
    ): array {
        $contextHours = max(1, (int) admin_setting('user_risk_review_context_hours', 24));
        $contextStartAt = now()->subHours($contextHours);
        $userSessions = $sessions->where('user_id', $user->id)->values();

        $matchedUsers = $users
            ->only($sessions->pluck('user_id')->unique()->values()->all())
            ->map(fn (User $matchedUser) => [
                'id' => (int) $matchedUser->id,
                'email' => (string) $matchedUser->email,
                'plan' => (string) ($matchedUser->plan?->name ?? '-'),
                'banned' => (bool) $matchedUser->banned,
            ])
            ->values()
            ->all();

        $recentAudits = AuditLog::query()
            ->where('user_id', $user->id)
            ->where('created_at', '>=', $contextStartAt)
            ->orderByDesc('created_at')
            ->limit(50)
            ->get();

        $trafficUsageLogs = UserTrafficUsageLog::query()
            ->with('node:id,name,protocol,location_name')
            ->where('user_id', $user->id)
            ->where('recorded_at', '>=', $contextStartAt->timestamp)
            ->orderByDesc('recorded_at')
            ->limit(200)
            ->get();

        $nodeTrafficRecords = NodeTrafficRecord::query()
            ->with('node:id,name,protocol,location_name')
            ->where('user_id', $user->id)
            ->where('record_date', '>=', $contextStartAt->copy()->startOfDay()->toDateString())
            ->orderByDesc('record_date')
            ->limit(100)
            ->get();

        $otherActiveIps = UserOnlineSession::query()
            ->where('user_id', $user->id)
            ->where('last_activity', '>=', now()->subMinutes(max(5, (int) admin_setting('user_risk_review_time_window_minutes', 60))))
            ->where('ip_address', '!=', $sharedIp)
            ->pluck('ip_address')
            ->unique()
            ->values()
            ->all();

        $sessionNodes = $userSessions
            ->map(fn (UserOnlineSession $session) => [
                'id' => (int) $session->node_id,
                'name' => (string) ($session->node?->name ?? '#'.$session->node_id),
                'protocol' => (string) ($session->node?->protocol ?? '-'),
                'location_name' => (string) ($session->node?->location_name ?? '-'),
                'connection_count' => (int) $session->connection_count,
                'last_activity' => optional($session->last_activity)->timestamp,
            ])
            ->values()
            ->all();

        $auditStats = [
            'total' => $recentAudits->count(),
            'blocked' => $recentAudits->where('action_taken', AuditLog::ACTION_BLOCKED)->count(),
            'allowed' => $recentAudits->where('action_taken', AuditLog::ACTION_ALLOWED)->count(),
            'logged' => $recentAudits->where('action_taken', AuditLog::ACTION_LOGGED)->count(),
            'shared_ip_hits' => $recentAudits->where('ip_address', $sharedIp)->count(),
            'top_domains' => $recentAudits
                ->pluck('target_domain')
                ->filter()
                ->countBy()
                ->sortDesc()
                ->take(5)
                ->toArray(),
        ];

        $trafficStats = [
            'raw_traffic_kb' => (int) $trafficUsageLogs->sum('raw_traffic_kb'),
            'billed_traffic_kb' => (int) $trafficUsageLogs->sum('billed_traffic_kb'),
            'usage_log_count' => $trafficUsageLogs->count(),
            'distinct_nodes' => $trafficUsageLogs->pluck('node_id')->unique()->count(),
            'top_nodes' => $trafficUsageLogs
                ->groupBy('node_id')
                ->map(function (Collection $nodeLogs, $nodeId) {
                    /** @var UserTrafficUsageLog|null $first */
                    $first = $nodeLogs->first();
                    return [
                        'node_id' => (int) $nodeId,
                        'node_name' => (string) ($first?->node?->name ?? '#'.$nodeId),
                        'billed_traffic_kb' => (int) $nodeLogs->sum('billed_traffic_kb'),
                    ];
                })
                ->sortByDesc('billed_traffic_kb')
                ->take(5)
                ->values()
                ->all(),
            'daily_node_traffic_kb' => $nodeTrafficRecords
                ->groupBy('record_date')
                ->map(fn (Collection $records) => (int) ($records->sum('upload_traffic') + $records->sum('download_traffic')))
                ->toArray(),
        ];

        return [
            'shared_ip' => $sharedIp,
            'matched_user_count' => $matchedUserCount,
            'matched_users' => $matchedUsers,
            'user' => [
                'id' => (int) $user->id,
                'email' => (string) $user->email,
                'plan' => (string) ($user->plan?->name ?? '-'),
                'current_shared_sessions' => $userSessions->count(),
                'shared_ip_connection_count' => (int) $userSessions->sum('connection_count'),
                'shared_ip_upload_traffic' => (int) $userSessions->sum('upload_traffic'),
                'shared_ip_download_traffic' => (int) $userSessions->sum('download_traffic'),
                'shared_ip_nodes' => $sessionNodes,
                'other_active_ips' => $otherActiveIps,
            ],
            'evidence' => [
                'context_hours' => $contextHours,
                'audit_stats' => $auditStats,
                'traffic_stats' => $trafficStats,
                'shared_ip_nodes' => $sessionNodes,
                'other_active_ip_count' => count($otherActiveIps),
            ],
        ];
    }

    private function reviewUserContext(array $reviewContext): array
    {
        $heuristic = $this->buildHeuristicResult($reviewContext);

        if (!$this->llmClient->isConfigured()) {
            return $heuristic;
        }

        try {
            $llmResult = $this->llmClient->reviewRisk(
                $this->buildSystemPrompt(),
                $this->buildUserPrompt($reviewContext)
            );

            $parsed = is_array($llmResult['parsed'] ?? null) ? $llmResult['parsed'] : [];
            $score = (int) ($parsed['suspicion_score'] ?? $heuristic['suspicion_score']);
            $score = max($heuristic['suspicion_score'], min(100, max(0, $score)));

            $riskLevel = strtolower((string) ($parsed['risk_level'] ?? $heuristic['risk_level']));
            if (!in_array($riskLevel, [
                UserRiskReview::LEVEL_LOW,
                UserRiskReview::LEVEL_MEDIUM,
                UserRiskReview::LEVEL_HIGH,
            ], true)) {
                $riskLevel = $heuristic['risk_level'];
            }
            if ($riskLevel === UserRiskReview::LEVEL_LOW) {
                $riskLevel = UserRiskReview::LEVEL_MEDIUM;
            }

            return [
                'risk_level' => $riskLevel,
                'suspicion_score' => $score,
                'summary' => trim((string) ($parsed['summary'] ?? $heuristic['summary'])) ?: $heuristic['summary'],
                'recommendation' => trim((string) ($parsed['recommendation'] ?? $heuristic['recommendation'])) ?: $heuristic['recommendation'],
                'llm_model' => (string) ($llmResult['model'] ?? $this->llmClient->getModel()),
                'raw_response' => is_string($llmResult['content'] ?? null)
                    ? $llmResult['content']
                    : json_encode($llmResult['raw'] ?? [], JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES),
            ];
        } catch (Throwable $e) {
            Log::warning('User risk review LLM fallback to heuristic', [
                'error' => $e->getMessage(),
                'user_id' => data_get($reviewContext, 'user.id'),
                'shared_ip' => data_get($reviewContext, 'shared_ip'),
            ]);

            return $heuristic;
        }
    }

    private function buildHeuristicResult(array $reviewContext): array
    {
        $matchedUserCount = (int) data_get($reviewContext, 'matched_user_count', 2);
        $auditCount = (int) data_get($reviewContext, 'evidence.audit_stats.total', 0);
        $blockedCount = (int) data_get($reviewContext, 'evidence.audit_stats.blocked', 0);
        $otherIpCount = (int) data_get($reviewContext, 'evidence.other_active_ip_count', 0);
        $billedTrafficKb = (int) data_get($reviewContext, 'evidence.traffic_stats.billed_traffic_kb', 0);

        $score = 50;
        $score += min(25, max(0, ($matchedUserCount - 1) * 10));
        $score += min(10, intdiv($auditCount, 20) * 5);
        $score += min(10, intdiv($blockedCount, 5) * 5);
        $score += $otherIpCount === 0 ? 5 : 0;
        $score += $billedTrafficKb >= 1024 * 1024 ? 10 : 0;
        $score = min(100, $score);

        $riskLevel = $score >= 80 ? UserRiskReview::LEVEL_HIGH : UserRiskReview::LEVEL_MEDIUM;
        $sharedIp = (string) data_get($reviewContext, 'shared_ip');
        $userId = (int) data_get($reviewContext, 'user.id');
        $userEmail = (string) data_get($reviewContext, 'user.email');

        return [
            'risk_level' => $riskLevel,
            'suspicion_score' => $score,
            'summary' => sprintf(
                '检测到用户 #%d (%s) 在共享 IP %s 上与 %d 个用户存在重叠使用行为，已按至少中风险处理。',
                $userId,
                $userEmail,
                $sharedIp,
                $matchedUserCount
            ),
            'recommendation' => $riskLevel === UserRiskReview::LEVEL_HIGH
                ? '建议立即人工复核该用户的最近节点、流量和审计行为，如确认异常可执行封禁。'
                : '建议人工复核该用户近期在线 IP、节点分布和审计行为，确认是否存在账号共享或滥用。',
            'llm_model' => null,
            'raw_response' => null,
        ];
    }

    private function buildSystemPrompt(): string
    {
        return implode("\n", [
            '你是代理节点滥用风控助手，需要根据共享 IP、流量和审计行为给出预审结论。',
            '共享 IP 命中多个用户时，最低风险等级不能低于 medium。',
            '请只返回 JSON，不要返回 Markdown。',
            'JSON 字段必须包含：risk_level, suspicion_score, summary, recommendation。',
            'risk_level 只能是 low、medium、high。',
            'suspicion_score 为 0 到 100 的整数。',
            'summary 和 recommendation 要用简体中文，简洁明确。',
        ]);
    }

    private function buildUserPrompt(array $reviewContext): string
    {
        try {
            return json_encode(
                $reviewContext,
                JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES | JSON_PRETTY_PRINT | JSON_THROW_ON_ERROR
            );
        } catch (Throwable $e) {
            Log::warning('User risk review context encoding failed', [
                'error' => $e->getMessage(),
                'user_id' => data_get($reviewContext, 'user.id'),
                'shared_ip' => data_get($reviewContext, 'shared_ip'),
            ]);

            return json_encode([
                'shared_ip' => data_get($reviewContext, 'shared_ip'),
                'matched_user_count' => data_get($reviewContext, 'matched_user_count'),
                'user' => data_get($reviewContext, 'user'),
                'evidence' => data_get($reviewContext, 'evidence'),
            ], JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES) ?: '{}';
        }
    }

    private function wasReviewedRecently(int $userId, string $sharedIp, int $cooldownMinutes): bool
    {
        $cutoff = time() - ($cooldownMinutes * 60);

        return UserRiskReview::query()
            ->where('user_id', $userId)
            ->where('shared_ip', $sharedIp)
            ->where('reviewed_at', '>=', $cutoff)
            ->exists();
    }

    private function sendTelegramAlert(string $sharedIp, int $matchedUserCount, Collection $reviews, Collection $users): int
    {
        if (!(bool) admin_setting('telegram_bot_enable', 0) || !(bool) admin_setting('telegram_notify_user_risk_detected', 1)) {
            return 0;
        }

        $recipients = User::query()
            ->where('is_super_admin', 1)
            ->whereNotNull('telegram_id')
            ->get(['id', 'email', 'telegram_id']);

        if ($recipients->isEmpty()) {
            return 0;
        }

        $lines = [
            '用户风险审查提醒',
            '共享 IP: ' . $sharedIp,
            '命中用户数: ' . $matchedUserCount,
            '审查记录数: ' . $reviews->count(),
            '',
        ];

        foreach ($reviews->take(8) as $review) {
            /** @var UserRiskReview $review */
            $reviewUser = $users->get($review->user_id);
            $lines[] = sprintf(
                '#%d %s | 风险=%s | 评分=%d',
                (int) $review->user_id,
                (string) ($reviewUser?->email ?? '-'),
                (string) $review->risk_level,
                (int) $review->suspicion_score
            );
            if ($review->summary) {
                $lines[] = '摘要: ' . $review->summary;
            }
            if ($review->recommendation) {
                $lines[] = '建议: ' . $review->recommendation;
            }
            $lines[] = '';
        }

        try {
            app(TelegramService::class)->queueMessageForUsers(
                $recipients,
                implode("\n", $lines),
                ''
            );

            return $recipients->count();
        } catch (Throwable) {
            return 0;
        }
    }
}
