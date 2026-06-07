#!/usr/bin/env python3
"""
Audit the frozen legacy PHP artisan command baseline against current Rust-owned
runtime equivalents.

This focuses on operational meaning, not strict 1:1 CLI UX parity:
- "covered" means the capability has a Rust runtime/API/scheduler equivalent
- "missing" means the capability is still PHP-only or manual-PHP oriented
- prefers `tools/compat_baselines/cli_commands.json` when present
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path


SIGNATURE_RE = re.compile(r"protected \$signature = '([^']+)'")

RUST_COVERAGE = {
    "backup:database": "backup_support::run_scheduled_database_backup + POST /api/v2/{admin_path}/system/runDatabaseBackup",
    "check:commission": "Rust scheduler: auto_check_commissions + auto_pay_commissions",
    "check:order": "Rust scheduler: check_orders",
    "check:server": "Rust scheduler: check_server_nodes_offline",
    "check:ticket": "Rust scheduler: check_tickets",
    "clear:user": "POST /api/v2/{admin_path}/system/cleanupDormantUsers",
    "cleanup:expired-node-sessions": "Rust scheduler: cleanup_expired_node_sessions",
    "cleanup:expired-online-status": "Rust scheduler: cleanup_expired_online_status",
    "hook:list": "GET /api/v2/{admin_path}/system/listHooks",
    "log:export": "GET /api/v2/{admin_path}/system/exportLogsCsv",
    "refunds:finalize-votes": "Rust scheduler: finalize_expired_refund_votings",
    "reset:password": "POST /api/v2/{admin_path}/system/resetUserPassword",
    "reset:log": "Rust scheduler: run_daily_log_cleanup",
    "reset:traffic": "Rust scheduler: initialize_missing_user_reset_times + reset_due_user_traffic",
    "reset:user": "POST /api/v2/{admin_path}/system/resetAllUserSecurity",
    "review:user-risk": "Rust scheduler: run_scheduled_risk_review + admin risk review APIs",
    "subscription:rotate-credentials": "Rust scheduler: rotate_subscription_credentials_daily",
    "send:remindMail": "Rust scheduler: run_send_remind_mail",
    "oauth:sync-linux-do-users": "Rust scheduler: run_linux_do_user_sync",
    "test": "Repository-level verification scripts: tools/verify_rust_default_stack_isolated.sh + tools/verify_rust_default_stack.sh + tools/bench_uniproxy_hotpaths.sh + tools/bench_legacy_submit.sh",
    "xboard:install": "Rust bootstrap APIs: /bootstrap/status /bootstrap/minimal /bootstrap/full",
    "xboard:statistics": "Rust scheduler: run_daily_statistics",
}
CLI_BASELINE_PATH = Path("tools/compat_baselines/cli_commands.json")


@dataclass(frozen=True)
class CliCommandAudit:
    signature: str
    source: str
    rust_coverage: str | None
    status: str


def parse_php_commands(root: Path) -> list[CliCommandAudit]:
    baseline = load_cli_baseline(root)
    if baseline is not None:
        return [
            CliCommandAudit(
                signature=item["signature"],
                source=item["source"],
                rust_coverage=RUST_COVERAGE.get(item["signature"].split()[0]),
                status="covered" if RUST_COVERAGE.get(item["signature"].split()[0]) else "missing",
            )
            for item in baseline
        ]

    records: list[CliCommandAudit] = []
    for path in sorted((root / "app" / "Console" / "Commands").glob("*.php")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        match = SIGNATURE_RE.search(text)
        if not match:
            continue
        signature = match.group(1)
        command = signature.split()[0]
        coverage = RUST_COVERAGE.get(command)
        records.append(
            CliCommandAudit(
                signature=signature,
                source=path.relative_to(root).as_posix(),
                rust_coverage=coverage,
                status="covered" if coverage else "missing",
            )
        )
    return records


def load_cli_baseline(root: Path) -> list[dict[str, str]] | None:
    path = root / CLI_BASELINE_PATH
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def build_report(root: Path) -> dict[str, object]:
    records = parse_php_commands(root)
    covered = [asdict(item) for item in records if item.status == "covered"]
    missing = [asdict(item) for item in records if item.status == "missing"]
    return {
        "php_command_count": len(records),
        "covered_count": len(covered),
        "missing_count": len(missing),
        "covered": covered,
        "missing": missing,
        "note": "Covered means the capability has a Rust runtime/API/scheduler equivalent; it does not guarantee identical artisan UX.",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit PHP artisan command surface against Rust equivalents")
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

    result = build_report(args.root.resolve())
    if args.format == "json":
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print(f"PHP artisan commands: {result['php_command_count']}")
        print(f"Covered by Rust:      {result['covered_count']}")
        print(f"Still PHP-only:       {result['missing_count']}")
        print()
        print("Still PHP-only commands:")
        for item in result["missing"]:
            print(f"  {item['signature']:<55} [{item['source']}]")
        print()
        print(result["note"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
