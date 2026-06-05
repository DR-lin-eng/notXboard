#!/usr/bin/env python3
"""
Audit scheduled Laravel console commands against the current Rust-owned
background scheduler implementation.

This focuses on the core command schedule declared in `app/Console/Kernel.php`.
Plugin-defined schedules remain a separate compatibility surface.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path


SCHEDULE_COMMAND_RE = re.compile(r"\$schedule->command\('([^']+)'")


@dataclass(frozen=True)
class ScheduledCommand:
    command: str
    source: str
    rust_coverage: str | None
    status: str


CORE_RUST_SCHEDULER_COVERAGE = {
    "xboard:statistics": "stats_support::run_daily_statistics",
    "check:order": "main.rs::check_orders",
    "check:commission": "main.rs::auto_check_commissions + main.rs::auto_pay_commissions",
    "check:ticket": "main.rs::check_tickets",
    "check:server": "main.rs::check_server_nodes_offline",
    "reset:traffic": "main.rs::initialize_missing_user_reset_times + main.rs::reset_due_user_traffic",
    "reset:log": "scheduler_support::run_daily_log_cleanup",
    "subscription:rotate-credentials": "main.rs::rotate_subscription_credentials_daily",
    "send:remindMail": "mail_reminder_support::run_send_remind_mail",
    "horizon:snapshot": "horizon_metrics_support::run_horizon_metrics_snapshot",
    "cleanup:expired-online-status": "main.rs::cleanup_expired_online_status",
    "cleanup:expired-node-sessions": "main.rs::cleanup_expired_node_sessions",
    "oauth:sync-linux-do-users": "oauth_sync_support::run_linux_do_user_sync",
    "refunds:finalize-votes": "main.rs::finalize_expired_refund_votings",
    "review:user-risk": "risk_review_support::run_scheduled_risk_review",
    "backup:database": "backup_support::run_scheduled_database_backup",
}


def parse_kernel_schedule(root: Path) -> list[str]:
    kernel = root / "app" / "Console" / "Kernel.php"
    commands: list[str] = []
    for line in kernel.read_text(encoding="utf-8", errors="ignore").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*") or stripped.startswith("*/"):
            continue
        match = SCHEDULE_COMMAND_RE.search(stripped)
        if match:
            commands.append(match.group(1))
    return commands


def audit_scheduler(root: Path) -> dict[str, object]:
    commands = parse_kernel_schedule(root)
    results: list[ScheduledCommand] = []
    for command in commands:
        rust_coverage = CORE_RUST_SCHEDULER_COVERAGE.get(command)
        results.append(
            ScheduledCommand(
                command=command,
                source="app/Console/Kernel.php",
                rust_coverage=rust_coverage,
                status="covered" if rust_coverage else "missing",
            )
        )

    missing = [asdict(item) for item in results if item.status == "missing"]
    covered = [asdict(item) for item in results if item.status == "covered"]
    return {
        "scheduled_command_count": len(results),
        "covered_count": len(covered),
        "missing_count": len(missing),
        "covered": covered,
        "missing": missing,
        "note": "PluginManager::registerPluginSchedules($schedule) is a separate compatibility surface and is not expanded here.",
    }


def print_text_report(result: dict[str, object]) -> None:
    print(f"Scheduled commands: {result['scheduled_command_count']}")
    print(f"Covered by Rust:    {result['covered_count']}")
    print(f"Still missing:      {result['missing_count']}")
    print()
    print("Covered commands:")
    for item in result["covered"]:
        print(f"  {item['command']:<32} -> {item['rust_coverage']}")
    print()
    print("Missing commands:")
    for item in result["missing"]:
        print(f"  {item['command']}")
    print()
    print(result["note"])


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit Laravel scheduler coverage in the Rust gateway")
    parser.add_argument(
        "--root",
        default=Path(__file__).resolve().parents[1],
        type=Path,
        help="Repo root path",
    )
    parser.add_argument(
        "--format",
        choices=("text", "json"),
        default="text",
        help="Output format",
    )
    args = parser.parse_args()

    result = audit_scheduler(args.root.resolve())
    if args.format == "json":
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print_text_report(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
