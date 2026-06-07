#!/usr/bin/env python3
"""
Audit HTTP route coverage between the frozen legacy route baseline and the
current Rust gateway registrations.

This is intentionally lightweight and repo-local:
- no third-party dependencies
- path/method coverage only
- treats `/api/v2/{admin_path}/...` as the normalized secure-admin surface
- prefers `tools/compat_baselines/php_routes.json.gz.b64` when present
"""

from __future__ import annotations

import base64
import argparse
import gzip
import json
import re
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


ROUTE_CALL_RE = re.compile(r"\$[A-Za-z_]\w*->(get|post|put|delete|any|match)\(")
ROUTE_PATH_RE = re.compile(r"\$[A-Za-z_]\w*->(?:get|post|put|delete|any|match)\(\s*'([^']*)'")
STRING_RE = re.compile(r"'([^']*)'")
PREFIX_VALUE_RE = re.compile(r"'prefix'\s*=>\s*(.+?)(?:,|\])")
RUST_ROUTE_PATH_RE = re.compile(r'\.route\("([^"]+)",')
ROUTE_BASELINE_PATH = Path("tools/compat_baselines/php_routes.json.gz.b64")


@dataclass(frozen=True)
class RouteRecord:
    method: str
    path: str
    source: str


def collapse_path(*segments: str) -> str:
    parts = []
    for segment in segments:
        for part in segment.split("/"):
            part = part.strip()
            if part:
                parts.append(part)
    return "/" + "/".join(parts)


def normalize_php_group_prefix(raw: str) -> str:
    raw = raw.strip()
    if "admin_setting(" in raw:
        return "{admin_path}"
    strings = [item.strip("/") for item in STRING_RE.findall(raw) if item.strip("/")]
    if not strings:
        return ""
    return "/".join(strings)


def normalize_php_route_path(version: str, prefixes: list[str], route_path: str) -> str:
    base = f"/api/{version.lower()}"
    suffix = collapse_path(*prefixes, route_path).strip("/")
    if not suffix:
        return base
    return f"{base}/{suffix}"


def parse_php_routes(root: Path) -> list[RouteRecord]:
    baseline = load_php_route_baseline(root)
    if baseline is not None:
        return baseline

    records: list[RouteRecord] = []
    for version_dir in sorted((root / "app" / "Http" / "Routes").iterdir()):
        if not version_dir.is_dir():
            continue
        version = version_dir.name
        for path in sorted(version_dir.glob("*.php")):
            records.extend(parse_php_route_file(root, version, path))
    return records


def load_php_route_baseline(root: Path) -> list[RouteRecord] | None:
    path = root / ROUTE_BASELINE_PATH
    if not path.is_file():
        return None

    encoded = "".join(line.strip() for line in path.read_text(encoding="utf-8").splitlines())
    payload = json.loads(gzip.decompress(base64.b64decode(encoded)).decode("utf-8"))
    return [
        RouteRecord(
            method=item["method"],
            path=item["path"],
            source=item["source"],
        )
        for item in payload
    ]


def parse_php_route_file(root: Path, version: str, path: Path) -> list[RouteRecord]:
    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    records: list[RouteRecord] = []
    group_stack: list[tuple[int, str]] = []
    pending_prefix: str | None = None
    capturing_group = False
    group_lines: list[str] = []
    brace_depth = 0

    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*") or stripped.startswith("*/"):
            continue

        if capturing_group:
            group_lines.append(stripped)
            if "function" in stripped:
                merged = " ".join(group_lines)
                match = PREFIX_VALUE_RE.search(merged)
                pending_prefix = normalize_php_group_prefix(match.group(1)) if match else ""
                capturing_group = False
                group_lines = []
                if "{" in stripped:
                    brace_depth += stripped.count("{") - stripped.count("}")
                    group_stack.append((brace_depth, pending_prefix))
                    pending_prefix = None
                    trim_group_stack(group_stack, brace_depth)
                continue
        elif "->group([" in stripped:
            group_lines = [stripped]
            if "function" in stripped:
                match = PREFIX_VALUE_RE.search(stripped)
                pending_prefix = normalize_php_group_prefix(match.group(1)) if match else ""
                if "{" in stripped:
                    brace_depth += stripped.count("{") - stripped.count("}")
                    group_stack.append((brace_depth, pending_prefix))
                    pending_prefix = None
                    trim_group_stack(group_stack, brace_depth)
                else:
                    capturing_group = True
                continue
            capturing_group = True
            continue

        route_call = ROUTE_CALL_RE.search(stripped)
        if route_call:
            method = route_call.group(1)
            route_path_match = ROUTE_PATH_RE.search(stripped)
            if route_path_match:
                route_path = route_path_match.group(1)
                prefix_chain = [prefix for _, prefix in group_stack if prefix]
                full_path = normalize_php_route_path(version, prefix_chain, route_path)
                methods = [method.upper()]
                if method == "any":
                    methods = ["ANY"]
                elif method == "match":
                    methods = [item.upper() for item in re.findall(r"'([A-Za-z]+)'", stripped)]
                for entry_method in methods:
                    records.append(
                        RouteRecord(
                            method=entry_method,
                            path=full_path,
                            source=path.relative_to(root).as_posix(),
                        )
                    )

        brace_depth += stripped.count("{") - stripped.count("}")
        trim_group_stack(group_stack, brace_depth)

    return records


def trim_group_stack(group_stack: list[tuple[int, str]], brace_depth: int) -> None:
    while group_stack and group_stack[-1][0] > brace_depth:
        group_stack.pop()


def parse_rust_routes(root: Path) -> list[RouteRecord]:
    records: list[RouteRecord] = []
    router_support = root / "rust-gateway" / "src" / "router_support.rs"
    admin_router = root / "rust-gateway" / "src" / "admin_v2" / "router.rs"

    records.extend(parse_rust_route_file(root, router_support, prefix=""))

    admin_relative = parse_rust_route_file(root, admin_router, prefix="")
    for route in admin_relative:
        records.append(
            RouteRecord(
                method=route.method,
                path=collapse_path("/api/v2/admin", route.path),
                source=route.source,
            )
        )
        records.append(
            RouteRecord(
                method=route.method,
                path=collapse_path("/api/v2/{admin_path}", route.path),
                source=route.source,
            )
        )

    deduped: dict[tuple[str, str, str], RouteRecord] = {}
    for record in records:
        deduped[(record.method, record.path, record.source)] = record
    return list(deduped.values())


def parse_rust_route_file(root: Path, path: Path, prefix: str) -> list[RouteRecord]:
    records: list[RouteRecord] = []
    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    for line in lines:
        match = RUST_ROUTE_PATH_RE.search(line)
        if not match:
            continue
        route_path = collapse_path(prefix, match.group(1))
        handler = line
        methods = set()
        for token, method in (
            ("get(", "GET"),
            ("post(", "POST"),
            ("put(", "PUT"),
            ("delete(", "DELETE"),
            ("any(", "ANY"),
        ):
            if token in handler:
                methods.add(method)
        if not methods:
            methods.add("UNKNOWN")
        for method in sorted(methods):
            records.append(
                RouteRecord(
                    method=method,
                    path=route_path,
                    source=path.relative_to(root).as_posix(),
                )
            )
    return records


def compare_routes(
    php_routes: Iterable[RouteRecord],
    rust_routes: Iterable[RouteRecord],
) -> dict[str, object]:
    php_routes = list(php_routes)
    rust_routes = list(rust_routes)
    rust_method_map: dict[str, set[str]] = defaultdict(set)
    rust_sources: dict[str, set[str]] = defaultdict(set)
    for route in rust_routes:
        rust_method_map[route.path].add(route.method)
        rust_sources[route.path].add(route.source)

    matched: list[dict[str, object]] = []
    missing: list[dict[str, object]] = []
    for route in php_routes:
        rust_methods = rust_method_map.get(route.path, set())
        if (
            route.method in rust_methods
            or "ANY" in rust_methods
            or (route.method == "ANY" and {"GET", "POST"}.issubset(rust_methods))
        ):
            matched.append(
                {
                    "method": route.method,
                    "path": route.path,
                    "php_source": route.source,
                    "rust_sources": sorted(rust_sources.get(route.path, set())),
                }
            )
        else:
            missing.append(asdict(route))

    rust_only = []
    php_keys = {(route.method, route.path) for route in php_routes}
    for route in rust_routes:
        if (route.method, route.path) not in php_keys and ("ANY", route.path) not in php_keys:
            rust_only.append(asdict(route))

    missing_by_source = Counter(item["source"] for item in missing)
    missing_by_prefix = Counter(classify_prefix(item["path"]) for item in missing)

    return {
        "php_route_count": len(php_routes),
        "rust_route_count": len(rust_routes),
        "matched_count": len(matched),
        "missing_count": len(missing),
        "rust_only_count": len(rust_only),
        "missing_by_source": dict(sorted(missing_by_source.items())),
        "missing_by_prefix": dict(sorted(missing_by_prefix.items())),
        "missing": missing,
        "matched": matched,
        "rust_only": rust_only,
    }


def classify_prefix(path: str) -> str:
    parts = [part for part in path.split("/") if part]
    if len(parts) >= 4 and parts[0] == "api":
        if parts[2] == "{admin_path}":
            return "/".join(parts[:4])
        return "/".join(parts[:4])
    if len(parts) >= 3:
        return "/".join(parts[:3])
    return path


def print_text_report(result: dict[str, object], limit: int) -> None:
    print(f"PHP routes:   {result['php_route_count']}")
    print(f"Rust routes:  {result['rust_route_count']}")
    print(f"Matched:      {result['matched_count']}")
    print(f"Missing:      {result['missing_count']}")
    print(f"Rust-only:    {result['rust_only_count']}")
    print()

    print("Missing by source:")
    for source, count in result["missing_by_source"].items():
        print(f"  {count:>3}  {source}")
    print()

    print("Missing by prefix:")
    for prefix, count in result["missing_by_prefix"].items():
        print(f"  {count:>3}  /{prefix}")
    print()

    if limit <= 0:
        return

    print(f"Top {min(limit, len(result['missing']))} missing routes:")
    for item in result["missing"][:limit]:
        print(f"  {item['method']:<6} {item['path']}  [{item['source']}]")


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit PHP route coverage in the Rust gateway")
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
    parser.add_argument(
        "--limit",
        type=int,
        default=50,
        help="Number of missing routes to print in text mode",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    php_routes = parse_php_routes(root)
    rust_routes = parse_rust_routes(root)
    result = compare_routes(php_routes, rust_routes)

    if args.format == "json":
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print_text_report(result, args.limit)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
