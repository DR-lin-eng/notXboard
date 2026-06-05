#!/usr/bin/env python3
"""
Audit plugin-defined scheduler hooks.

This checks whether any plugin class currently overrides `AbstractPlugin::schedule`.
If no plugin overrides it, the Laravel plugin scheduler compatibility surface is
effectively dormant for the current built-in plugin set.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


SCHEDULE_OVERRIDE_RE = re.compile(r"function\s+schedule\s*\(")


def audit_plugin_schedules(root: Path) -> dict[str, object]:
    plugin_files = sorted((root / "plugins").glob("*/Plugin.php"))
    overrides = []
    for path in plugin_files:
        text = path.read_text(encoding="utf-8", errors="ignore")
        if SCHEDULE_OVERRIDE_RE.search(text):
            overrides.append(path.relative_to(root).as_posix())
    return {
        "plugin_count": len(plugin_files),
        "schedule_override_count": len(overrides),
        "schedule_overrides": overrides,
        "status": "dormant" if not overrides else "active",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit plugin scheduler overrides")
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

    result = audit_plugin_schedules(args.root.resolve())
    if args.format == "json":
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print(f"Plugins scanned:           {result['plugin_count']}")
        print(f"Schedule overrides found:  {result['schedule_override_count']}")
        print(f"Compatibility surface:     {result['status']}")
        if result["schedule_overrides"]:
            print()
            for item in result["schedule_overrides"]:
                print(f"  {item}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
