#!/usr/bin/env python3
"""Fail when MarkShareX source-of-truth facts drift from public documentation."""

from __future__ import annotations

import argparse
from collections import Counter
import json
import re
import sqlite3
import sys
from pathlib import Path
from typing import List, Sequence, Set, Tuple


DATABASE_SUPPORT_STATEMENT = (
    "当前版本仅支持 SQLite；PostgreSQL 和 MySQL 计划在后续版本支持。"
)
STALE_DATABASE_CLAIMS = (
    "SQLite（可选 PostgreSQL）",
    "SQLite/PG",
    "SeaORM 同时编译了 PostgreSQL 驱动",
    "Cargo 虽启用了 PostgreSQL 驱动",
    "无缝切换 PostgreSQL",
)


def read(root: Path, relative: str) -> str:
    path = root / relative
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ValueError(f"required file {relative} is unreadable: {error}") from error


def cargo_package_version(cargo: str) -> str:
    package = re.search(r"(?ms)^\[package\]\s*(.*?)(?=^\[|\Z)", cargo)
    if not package:
        raise ValueError("Cargo.toml has no [package] section")
    version = re.search(r'^version\s*=\s*"([^"]+)"\s*$', package.group(1), re.M)
    if not version:
        raise ValueError("Cargo.toml [package] has no version")
    return version.group(1)


def _strip_toml_comment(line: str) -> str:
    quote = None
    escaped = False
    for index, character in enumerate(line):
        if escaped:
            escaped = False
            continue
        if character == "\\" and quote == '"':
            escaped = True
            continue
        if character in {'"', "'"}:
            quote = None if quote == character else character if quote is None else quote
            continue
        if character == "#" and quote is None:
            return line[:index]
    return line


def cargo_sea_orm_features(cargo: str) -> Set[str]:
    """Parse the sea-orm feature array without treating TOML comments as features."""
    dependency_lines: List[str] = []
    in_dependencies = False
    for raw_line in cargo.splitlines():
        line = _strip_toml_comment(raw_line).strip()
        if re.fullmatch(r"\[[^]]+\]", line):
            in_dependencies = line == "[dependencies]"
            continue
        if in_dependencies:
            dependency_lines.append(line)

    assignment: List[str] = []
    collecting = False
    brace_depth = 0
    for line in dependency_lines:
        if not collecting:
            match = re.match(r"^sea-orm\s*=\s*(.*)$", line)
            if not match:
                continue
            collecting = True
            line = match.group(1)
        assignment.append(line)
        brace_depth += line.count("{") - line.count("}")
        if brace_depth <= 0:
            break
    if not assignment:
        raise ValueError("Cargo.toml dependencies has no sea-orm entry")

    value = "\n".join(assignment)
    if not value.lstrip().startswith("{"):
        return set()
    feature_match = re.search(r"\bfeatures\s*=\s*\[(.*?)]", value, re.S)
    if not feature_match:
        return set()
    feature_source = feature_match.group(1)
    features = re.findall(r'["\']([^"\']+)["\']', feature_source)
    remainder = re.sub(r'["\'][^"\']+["\']', "", feature_source)
    if remainder.replace(",", "").strip():
        raise ValueError("Cargo.toml sea-orm features must contain quoted strings")
    return set(features)


def endpoint_operations(source: str) -> List[Tuple[str, str]]:
    operations = re.findall(
        r'^\s*\(([A-Z][A-Z0-9_]*),\s*"(/api/v1/[^"]*)"\s*,',
        source,
        re.M,
    )
    return [("POST" if method == "POST_CSP" else method, path) for method, path in operations]


def environment_bindings(source: str) -> List[Tuple[str, str]]:
    marker = "const ENVIRONMENT_BINDINGS:"
    if marker not in source:
        raise ValueError("src/config/mod.rs has no ENVIRONMENT_BINDINGS table")
    block = source.split(marker, 1)[1].split("];", 1)[0]
    return re.findall(
        r'name:\s*"(MARKSHAREX_[A-Z0-9_]+)"\s*,\s*'
        r'path:\s*"([a-z0-9_.]+)"',
        block,
        re.S,
    )


def env_example_names(source: str) -> Set[str]:
    names: Set[str] = set()
    for raw_line in source.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        name, separator, _ = line.partition("=")
        if separator:
            names.add(name)
    return names


def workflow_trigger_names(source: str) -> Set[str]:
    """Read direct children of the workflow's top-level ``on`` mapping."""
    lines = source.splitlines()
    for index, raw_line in enumerate(lines):
        if raw_line.startswith((" ", "\t")) or raw_line.strip() != "on:":
            continue
        triggers: Set[str] = set()
        child_indent = None
        for candidate in lines[index + 1 :]:
            stripped = candidate.strip()
            if not stripped or stripped.startswith("#"):
                continue
            indent = len(candidate) - len(candidate.lstrip(" "))
            if indent == 0:
                break
            if child_indent is None:
                child_indent = indent
            if indent != child_indent:
                continue
            match = re.match(r"^([A-Za-z_][A-Za-z0-9_-]*):(?:\s.*)?$", stripped)
            if match:
                triggers.add(match.group(1))
        return triggers
    return set()


def _indent(raw_line: str) -> int:
    return len(raw_line) - len(raw_line.lstrip(" "))


def _yaml_value(content: str, key: str) -> str | None:
    prefix = f"{key}:"
    if not content.startswith(prefix):
        return None
    value = content[len(prefix) :].strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {'"', "'"}:
        value = value[1:-1]
    return value


def workflow_job_contract(source: str) -> Tuple[Set[str], List[str]]:
    """Extract run commands and actions only from jobs and their executable steps."""
    lines = source.splitlines()
    jobs_index = next(
        (
            index
            for index, raw_line in enumerate(lines)
            if _indent(raw_line) == 0 and raw_line.strip() == "jobs:"
        ),
        None,
    )
    if jobs_index is None:
        return set(), []

    jobs_end = len(lines)
    for index in range(jobs_index + 1, len(lines)):
        if lines[index].strip() and _indent(lines[index]) == 0:
            jobs_end = index
            break
    job_indent = next(
        (
            _indent(lines[index])
            for index in range(jobs_index + 1, jobs_end)
            if lines[index].strip() and not lines[index].lstrip().startswith("#")
        ),
        None,
    )
    if job_indent is None:
        return set(), []

    commands: Set[str] = set()
    actions: List[str] = []
    job_starts = [
        index
        for index in range(jobs_index + 1, jobs_end)
        if _indent(lines[index]) == job_indent
        and re.match(r"^[A-Za-z_][A-Za-z0-9_-]*:\s*$", lines[index].strip())
    ]
    for position, job_start in enumerate(job_starts):
        job_end = job_starts[position + 1] if position + 1 < len(job_starts) else jobs_end
        property_indent = next(
            (
                _indent(lines[index])
                for index in range(job_start + 1, job_end)
                if lines[index].strip() and not lines[index].lstrip().startswith("#")
            ),
            None,
        )
        if property_indent is None:
            continue
        steps_index = None
        job_conditional = False
        for index in range(job_start + 1, job_end):
            if _indent(lines[index]) != property_indent:
                continue
            content = lines[index].strip()
            action = _yaml_value(content, "uses")
            if action is not None:
                actions.append(action.split(" #", 1)[0].strip())
            if _yaml_value(content, "if") is not None:
                job_conditional = True
            if content == "steps:":
                steps_index = index
        if steps_index is None:
            continue

        step_indent = next(
            (
                _indent(lines[index])
                for index in range(steps_index + 1, job_end)
                if lines[index].lstrip().startswith("-")
                and not lines[index].lstrip().startswith("#")
            ),
            None,
        )
        if step_indent is None:
            continue
        step_starts = [
            index
            for index in range(steps_index + 1, job_end)
            if _indent(lines[index]) == step_indent and lines[index].lstrip().startswith("-")
        ]
        for step_position, step_start in enumerate(step_starts):
            step_end = (
                step_starts[step_position + 1]
                if step_position + 1 < len(step_starts)
                else job_end
            )
            first_content = lines[step_start].strip()[1:].strip()
            property_lines = [(step_start, first_content)]
            child_indent = next(
                (
                    _indent(lines[index])
                    for index in range(step_start + 1, step_end)
                    if lines[index].strip() and not lines[index].lstrip().startswith("#")
                ),
                None,
            )
            if child_indent is not None:
                property_lines.extend(
                    (index, lines[index].strip())
                    for index in range(step_start + 1, step_end)
                    if _indent(lines[index]) == child_indent
                )
            step_conditional = any(
                _yaml_value(content, "if") is not None for _, content in property_lines
            )
            for line_index, content in property_lines:
                action = _yaml_value(content, "uses")
                if action is not None:
                    actions.append(action.split(" #", 1)[0].strip())
                run = _yaml_value(content, "run")
                if run is None or job_conditional or step_conditional:
                    continue
                if run in {"|", "|-", "|+", ">", ">-", ">+"}:
                    run_indent = _indent(lines[line_index])
                    for index in range(line_index + 1, step_end):
                        candidate = lines[index].strip()
                        if candidate and _indent(lines[index]) <= run_indent:
                            break
                        if candidate and not candidate.startswith("#"):
                            commands.add(candidate)
                elif run:
                    commands.add(run.split(" #", 1)[0].strip())
    return commands, actions


def unpinned_remote_actions(actions: Sequence[str]) -> List[str]:
    unpinned: List[str] = []
    for action in actions:
        if action.startswith(("./", "docker://")):
            continue
        _, separator, reference = action.rpartition("@")
        if not separator or not re.fullmatch(r"[0-9a-fA-F]{40}", reference):
            unpinned.append(action)
    return unpinned


def schema_counts(schema: str) -> Tuple[int, int]:
    database = sqlite3.connect(":memory:")
    try:
        database.executescript(schema)
        tables = database.execute(
            "SELECT COUNT(*) FROM sqlite_master "
            "WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        ).fetchone()[0]
        indexes = database.execute(
            "SELECT COUNT(*) FROM sqlite_master "
            "WHERE type='index' AND name NOT LIKE 'sqlite_%'"
        ).fetchone()[0]
        return int(tables), int(indexes)
    finally:
        database.close()


def check_repository(root: Path) -> List[str]:
    errors: List[str] = []
    try:
        cargo = read(root, "Cargo.toml")
        frontend_package = json.loads(read(root, "frontend/package.json"))
        readme = read(root, "README.md")
        system_doc = read(root, "docs/MarkShareX系统全貌.md")
        config_doc = read(root, "docs/CONFIG.md")
        env_example = read(root, ".env.example")
        config_source = read(root, "src/config/mod.rs")
        endpoint_source = read(root, "src/api_endpoints.rs")
        endpoint_descriptions = read(root, "src/endpoint_descriptions.tsv")
        api_doc_source = read(root, "src/api_doc.rs")
        initial_schema = read(root, "migrations/0000000000_init_schema.sql")
    except (ValueError, json.JSONDecodeError) as error:
        return [f"input: {error}"]

    try:
        cargo_version = cargo_package_version(cargo)
    except ValueError as error:
        errors.append(f"version: {error}")
        cargo_version = ""
    frontend_version = str(frontend_package.get("version", ""))
    if cargo_version != frontend_version:
        errors.append(
            "version: Cargo.toml "
            f"({cargo_version}) != frontend/package.json ({frontend_version})"
        )
    expected_badge = f"version-{cargo_version}-blue.svg"
    if cargo_version and expected_badge not in readme:
        errors.append(f"version: README badge must contain {expected_badge}")
    if 'env!("CARGO_PKG_VERSION")' not in api_doc_source:
        errors.append("version: OpenAPI version must derive from CARGO_PKG_VERSION")
    expected_system_version = f"*本文档按 MarkShareX v{cargo_version} 当前源码整理。"
    if cargo_version and expected_system_version not in system_doc:
        errors.append(
            f"version: system document must contain {expected_system_version}"
        )

    try:
        database_features = cargo_sea_orm_features(cargo)
    except ValueError as error:
        errors.append(f"database: {error}")
        database_features = set()
    if "sqlx-sqlite" not in database_features:
        errors.append("database: Cargo.toml must enable sqlx-sqlite")
    for unsupported_feature in ("sqlx-postgres", "sqlx-mysql"):
        if unsupported_feature in database_features:
            errors.append(
                f"database: current release must not enable {unsupported_feature}"
            )
    for relative, document in (
        ("README.md", readme),
        ("docs/MarkShareX系统全貌.md", system_doc),
    ):
        if DATABASE_SUPPORT_STATEMENT not in document:
            errors.append(
                f"database: {relative} must state: {DATABASE_SUPPORT_STATEMENT}"
            )
        for stale_claim in STALE_DATABASE_CLAIMS:
            if stale_claim in document:
                errors.append(
                    f"database: {relative} contains stale claim {stale_claim!r}"
                )

    operations = endpoint_operations(endpoint_source)
    unique_operations = set(operations)
    if not operations:
        errors.append("endpoint: no /api/v1 operations found in authoritative catalog")
    if len(unique_operations) != len(operations):
        errors.append("endpoint: authoritative catalog contains duplicate method/path pairs")
    operation_count = len(operations)
    readme_count = f"共 {operation_count} 个 REST API operation。"
    system_count = (
        f"`src/api_endpoints.rs` 是 {operation_count} 个 `/api/v1/*` operation 的权威目录。"
    )
    if readme_count not in readme:
        errors.append(f"endpoint: README.md must contain {readme_count}")
    if system_count not in system_doc:
        errors.append(f"endpoint: system document must contain {system_count}")

    description_operations: Set[Tuple[str, str]] = set()
    for line_number, line in enumerate(endpoint_descriptions.splitlines(), 1):
        if not line.strip():
            continue
        fields = line.split("\t", 2)
        if len(fields) != 3:
            errors.append(
                f"endpoint: endpoint_descriptions.tsv line {line_number} is malformed"
            )
            continue
        operation = (fields[0], fields[1])
        if operation in description_operations:
            errors.append(
                "endpoint: endpoint_descriptions.tsv contains duplicate "
                f"{fields[0]} {fields[1]} at line {line_number}"
            )
        description_operations.add(operation)
    missing_descriptions = unique_operations - description_operations
    extra_descriptions = description_operations - unique_operations
    if missing_descriptions or extra_descriptions:
        errors.append(
            "endpoint: description catalog differs from endpoint catalog "
            f"(missing={len(missing_descriptions)}, extra={len(extra_descriptions)})"
        )

    try:
        binding_entries = environment_bindings(config_source)
    except ValueError as error:
        errors.append(f"environment: {error}")
        binding_entries = []
    duplicate_names = sorted(
        name for name, count in Counter(name for name, _ in binding_entries).items() if count > 1
    )
    duplicate_paths = sorted(
        path for path, count in Counter(path for _, path in binding_entries).items() if count > 1
    )
    if duplicate_names or duplicate_paths:
        errors.append(
            "environment: ENVIRONMENT_BINDINGS contains duplicates "
            f"(names={duplicate_names}, paths={duplicate_paths})"
        )
    bindings = set(binding_entries)
    supported_names = {name for name, _ in bindings}
    documented_names = env_example_names(env_example)
    if documented_names != supported_names:
        errors.append(
            "environment: .env.example differs from ENVIRONMENT_BINDINGS "
            f"(missing={sorted(supported_names - documented_names)}, "
            f"extra={sorted(documented_names - supported_names)})"
        )
    for name, path in sorted(bindings):
        if f"`{name}` | `{path}`" not in config_doc:
            errors.append(f"environment: docs/CONFIG.md is missing {name} -> {path}")
    expected_environment_count = f"完整的 {len(bindings)} 个环境变量"
    if expected_environment_count not in system_doc:
        errors.append(
            "environment: system document must contain "
            f"{expected_environment_count}"
        )
    documented_environment_counts = {
        int(value)
        for value in re.findall(r"完整的\s*(\d+)\s*个环境变量", system_doc)
    }
    conflicting_environment_counts = documented_environment_counts - {len(bindings)}
    if conflicting_environment_counts:
        errors.append(
            "environment: system document contains conflicting environment counts "
            f"{sorted(conflicting_environment_counts)}; expected {len(bindings)}"
        )

    try:
        table_count, index_count = schema_counts(initial_schema)
    except sqlite3.Error as error:
        errors.append(f"schema: initial migration does not execute in SQLite: {error}")
    else:
        schema_summary = (
            f"初始化 Schema 当前包含 {table_count} 张应用表和 "
            f"{index_count} 个应用索引。"
        )
        if schema_summary not in system_doc:
            errors.append(f"schema: system document must contain {schema_summary}")
        readme_schema_summary = f"共 {table_count} 张应用表："
        if readme_schema_summary not in readme:
            errors.append(f"schema: README.md must contain {readme_schema_summary}")
        documented_readme_table_counts = {
            int(value)
            for value in re.findall(r"共\s*(\d+)\s*张(?:应用)?表", readme)
        }
        conflicting_readme_table_counts = documented_readme_table_counts - {table_count}
        if conflicting_readme_table_counts:
            errors.append(
                "schema: README.md contains conflicting table counts "
                f"{sorted(conflicting_readme_table_counts)}; expected {table_count}"
            )
        documented_table_counts = {
            int(value)
            for value in re.findall(r"初始化 Schema[^\n]*?(\d+) 张", system_doc)
        }
        conflicting_table_counts = documented_table_counts - {table_count}
        if conflicting_table_counts:
            errors.append(
                "schema: system document contains conflicting initialized table counts "
                f"{sorted(conflicting_table_counts)}; expected {table_count}"
            )
        documented_index_counts = {
            int(value)
            for value in re.findall(r"初始化 Schema[^\n]*?(\d+) 个应用索引", system_doc)
        }
        conflicting_index_counts = documented_index_counts - {index_count}
        if conflicting_index_counts:
            errors.append(
                "schema: system document contains conflicting initialized index counts "
                f"{sorted(conflicting_index_counts)}; expected {index_count}"
            )

    try:
        workflow = read(root, ".github/workflows/documentation-sync.yml")
    except ValueError as error:
        errors.append(f"ci: {error}")
    else:
        triggers = workflow_trigger_names(workflow)
        missing_triggers = {"push", "pull_request"} - triggers
        if missing_triggers:
            errors.append(
                "ci: documentation sync workflow is missing triggers "
                f"{sorted(missing_triggers)}"
            )
        executable_commands, actions = workflow_job_contract(workflow)
        unpinned_actions = unpinned_remote_actions(actions)
        if unpinned_actions:
            errors.append(
                "ci: remote actions must be pinned to full commit SHAs "
                f"{sorted(unpinned_actions)}"
            )
        required_ci_commands = (
            "python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v",
            "python3 scripts/check_doc_sync.py",
            "cargo test --lib",
            "cargo test --test router_factory",
        )
        for command in required_ci_commands:
            if command not in executable_commands:
                errors.append(f"ci: documentation sync workflow is missing {command}")

    return errors


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="repository root (defaults to the parent of scripts/)",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    errors = check_repository(args.root.resolve())
    if errors:
        print("[doc-sync] FAIL")
        for error in errors:
            print(f"- {error}")
        return 1
    print("[doc-sync] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
