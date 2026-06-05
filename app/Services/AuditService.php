<?php

namespace App\Services;

use App\Models\AuditLog;
use App\Models\AuditRule;
use App\Models\ServerNode;

class AuditService
{
    /**
     * Evaluate audit rules for a given target and create an AuditLog when matched.
     *
     * If no rule matches, returns null (treated as allowed).
     */
    public function evaluateAndLog(
        ServerNode $node,
        int $userId,
        string $ipAddress,
        ?string $targetDomain = null,
        ?string $targetProtocol = null
    ): ?AuditLog {
        $targetDomain = $targetDomain !== null ? trim($targetDomain) : null;
        $targetProtocol = $targetProtocol !== null ? trim($targetProtocol) : null;

        /** @var AuditRule|null $matchedRule */
        $matchedRule = null;

        $rules = AuditRule::query()
            ->where('node_id', $node->id)
            ->where('is_active', 1)
            ->orderBy('id')
            ->get();

        foreach ($rules as $rule) {
            $target = match ($rule->rule_type) {
                AuditRule::TYPE_DOMAIN => $targetDomain,
                AuditRule::TYPE_PROTOCOL => $targetProtocol,
                AuditRule::TYPE_IP => $ipAddress,
                default => null,
            };

            if ($target === null || $target === '') {
                continue;
            }

            if ($rule->matches($target)) {
                $matchedRule = $rule;
                break;
            }
        }

        if (!$matchedRule) {
            return null;
        }

        $actionTaken = match ($matchedRule->action) {
            AuditRule::ACTION_BLOCK => AuditLog::ACTION_BLOCKED,
            AuditRule::ACTION_ALLOW => AuditLog::ACTION_ALLOWED,
            AuditRule::ACTION_LOG => AuditLog::ACTION_LOGGED,
            default => AuditLog::ACTION_LOGGED,
        };

        return AuditLog::createLog(
            userId: $userId,
            nodeId: $node->id,
            ipAddress: $ipAddress,
            actionTaken: $actionTaken,
            ruleId: $matchedRule->id,
            targetDomain: $targetDomain,
            targetProtocol: $targetProtocol
        );
    }
}

