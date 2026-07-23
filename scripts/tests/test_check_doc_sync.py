from __future__ import annotations

import json
import sqlite3
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "check_doc_sync.py"
DATABASE_SUPPORT_STATEMENT = (
    "当前版本仅支持 SQLite；PostgreSQL 和 MySQL 计划在后续版本支持。"
)


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")


def create_repository(root: Path) -> None:
    write(
        root / "Cargo.toml",
        """
        [package]
        name = "marksharex"
        version = "1.2.3"

        [dependencies]
        sea-orm = { version = "1", features = ["sqlx-sqlite"] }
        """,
    )
    write(
        root / "frontend/package.json",
        json.dumps({"name": "marksharex-frontend", "version": "1.2.3"}),
    )
    write(
        root / "README.md",
        f"""
        [![Version](https://img.shields.io/badge/version-1.2.3-blue.svg)](https://example.test)

        {DATABASE_SUPPORT_STATEMENT}

        共 2 个 REST API operation。

        共 1 张应用表：
        """,
    )
    write(
        root / "docs/MarkShareX系统全貌.md",
        f"""
        # MarkShareX 系统全貌

        {DATABASE_SUPPORT_STATEMENT}

        `src/api_endpoints.rs` 是 2 个 `/api/v1/*` operation 的权威目录。

        初始化 Schema 当前包含 1 张应用表和 1 个应用索引。

        完整的 1 个环境变量。

        *本文档按 MarkShareX v1.2.3 当前源码整理。*
        """,
    )
    write(
        root / "docs/CONFIG.md",
        """
        | 环境变量 | TOML 路径 |
        |---|---|
        | `MARKSHAREX_SERVER_PORT` | `server.port` |
        """,
    )
    write(root / ".env.example", "MARKSHAREX_SERVER_PORT=5023\n")
    write(
        root / "src/config/mod.rs",
        """
        const ENVIRONMENT_BINDINGS: &[EnvironmentBinding] = &[
            EnvironmentBinding {
                name: "MARKSHAREX_SERVER_PORT",
                path: "server.port",
                kind: EnvironmentValueKind::U16,
            },
        ];
        """,
    )
    write(
        root / "src/api_endpoints.rs",
        """
        macro_rules! api_endpoint_catalog {
            ($callback:ident) => {
                $callback! {
                    ;
                    (GET, "/api/v1/health", health, crate::__path_health, false),
                    (POST, "/api/v1/posts", create_post, crate::__path_create_post, true)
                }
            };
        }
        """,
    )
    write(
        root / "src/endpoint_descriptions.tsv",
        "GET\t/api/v1/health\tHealth\nPOST\t/api/v1/posts\tCreate post\n",
    )
    write(
        root / "src/api_doc.rs",
        'document.info.version = env!("CARGO_PKG_VERSION").to_string();\n',
    )
    write(
        root / ".github/workflows/documentation-sync.yml",
        """
        name: Documentation Sync
        on:
          push:
          pull_request:
        jobs:
          documentation-sync:
            steps:
              - uses: actions/checkout@0000000000000000000000000000000000000000 # v4
              - run: python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v
              - run: python3 scripts/check_doc_sync.py
              - run: cargo test --lib
              - run: cargo test --test router_factory
        """,
    )
    schema = """
        CREATE TABLE example (id INTEGER PRIMARY KEY);
        CREATE INDEX idx_example_id ON example(id);
    """
    write(root / "migrations/0000000000_init_schema.sql", schema)
    db = sqlite3.connect(":memory:")
    db.executescript(textwrap.dedent(schema))
    db.close()


def run_checker(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--root", str(root)],
        text=True,
        capture_output=True,
        check=False,
    )


class DocumentationSyncCheckerTests(unittest.TestCase):
    def test_synchronized_repository_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            result = run_checker(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_version_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            package = json.loads((root / "frontend/package.json").read_text())
            package["version"] = "9.9.9"
            write(root / "frontend/package.json", json.dumps(package))
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("version", result.stdout.lower())

    def test_readme_badge_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            readme = root / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8").replace("version-1.2.3", "version-1.2.2"),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("version", result.stdout.lower())

    def test_missing_sqlite_feature_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            cargo = root / "Cargo.toml"
            cargo.write_text(
                cargo.read_text(encoding="utf-8").replace(
                    'features = ["sqlx-sqlite"]', 'features = []'
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("database", result.stdout.lower())

    def test_enabled_non_sqlite_feature_fails(self) -> None:
        for feature in ("sqlx-postgres", "sqlx-mysql"):
            with self.subTest(feature=feature), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                create_repository(root)
                cargo = root / "Cargo.toml"
                cargo.write_text(
                    cargo.read_text(encoding="utf-8").replace(
                        'features = ["sqlx-sqlite"]',
                        f'features = ["sqlx-sqlite", "{feature}"]',
                    ),
                    encoding="utf-8",
                )
                result = run_checker(root)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("database", result.stdout.lower())

    def test_database_feature_name_in_comment_is_ignored(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            cargo = root / "Cargo.toml"
            cargo.write_text(
                cargo.read_text(encoding="utf-8")
                + "\n# future: sqlx-postgres and sqlx-mysql\n",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_database_support_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            write(root / "README.md", "SQLite（可选 PostgreSQL）\n")
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("database", result.stdout.lower())

    def test_endpoint_count_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8").replace(
                    "是 2 个 `/api/v1/*` operation", "是 3 个 `/api/v1/*` operation"
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("endpoint", result.stdout.lower())

    def test_new_http_method_is_counted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            endpoints = root / "src/api_endpoints.rs"
            endpoints.write_text(
                endpoints.read_text(encoding="utf-8").replace(
                    "(POST, \"/api/v1/posts\", create_post, crate::__path_create_post, true)",
                    "(POST, \"/api/v1/posts\", create_post, crate::__path_create_post, true),\n"
                    "                    (PATCH, \"/api/v1/posts/:id\", patch_post, crate::__path_patch_post, true)",
                ),
                encoding="utf-8",
            )
            descriptions = root / "src/endpoint_descriptions.tsv"
            descriptions.write_text(
                descriptions.read_text(encoding="utf-8")
                + "PATCH\t/api/v1/posts/:id\tPatch post\n",
                encoding="utf-8",
            )
            readme = root / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8").replace(
                    "共 2 个 REST API operation。", "共 3 个 REST API operation。"
                ),
                encoding="utf-8",
            )
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8").replace("是 2 个", "是 3 个"),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_environment_mapping_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            write(root / ".env.example", "MARKSHAREX_UNKNOWN=value\n")
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("environment", result.stdout.lower())

    def test_duplicate_environment_binding_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            config = root / "src/config/mod.rs"
            config.write_text(
                config.read_text(encoding="utf-8").replace(
                    "];",
                    """    EnvironmentBinding {
        name: \"MARKSHAREX_SERVER_PORT\",
        path: \"server.port\",
        kind: EnvironmentValueKind::U16,
    },
];""",
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("environment", result.stdout.lower())

    def test_duplicate_environment_name_or_path_fails(self) -> None:
        mutations = (
            ("MARKSHAREX_SERVER_PORT", "server.other_port"),
            ("MARKSHAREX_SERVER_OTHER_PORT", "server.port"),
        )
        for name, path in mutations:
            with self.subTest(name=name, path=path), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                create_repository(root)
                config = root / "src/config/mod.rs"
                config.write_text(
                    config.read_text(encoding="utf-8").replace(
                        "];",
                        f'''    EnvironmentBinding {{
        name: "{name}",
        path: "{path}",
        kind: EnvironmentValueKind::U16,
    }},
];''',
                    ),
                    encoding="utf-8",
                )
                result = run_checker(root)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("environment", result.stdout.lower())

    def test_system_document_environment_count_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8").replace(
                    "完整的 1 个环境变量", "完整的 2 个环境变量"
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("environment", result.stdout.lower())

    def test_conflicting_system_document_environment_count_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8")
                + "\n完整的 2 个环境变量。\n",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("environment", result.stdout.lower())

    def test_missing_ci_workflow_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            (root / ".github/workflows/documentation-sync.yml").unlink()
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("ci", result.stdout.lower())

    def test_missing_ci_command_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            workflow = root / ".github/workflows/documentation-sync.yml"
            workflow.write_text(
                workflow.read_text(encoding="utf-8").replace(
                    "- run: cargo test --test router_factory\n", ""
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("ci", result.stdout.lower())

    def test_missing_push_and_pull_request_triggers_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            workflow = root / ".github/workflows/documentation-sync.yml"
            workflow.write_text(
                workflow.read_text(encoding="utf-8").replace(
                    "on:\n  push:\n  pull_request:\n",
                    "on:\n  workflow_dispatch:\n",
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("ci", result.stdout.lower())

    def test_each_required_trigger_is_checked(self) -> None:
        for trigger in ("push", "pull_request"):
            with self.subTest(trigger=trigger), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                create_repository(root)
                workflow = root / ".github/workflows/documentation-sync.yml"
                workflow.write_text(
                    workflow.read_text(encoding="utf-8").replace(f"  {trigger}:\n", ""),
                    encoding="utf-8",
                )
                result = run_checker(root)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("ci", result.stdout.lower())

    def test_commands_outside_job_steps_do_not_satisfy_ci_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            workflow = root / ".github/workflows/documentation-sync.yml"
            workflow.write_text(
                """name: Documentation Sync
on:
  push:
  pull_request:
jobs:
  documentation-sync:
    steps:
      - run: true
metadata:
  run: python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v
  nested:
    run: python3 scripts/check_doc_sync.py
    deeper:
      run: cargo test --lib
      last:
        run: cargo test --test router_factory
""",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("ci", result.stdout.lower())

    def test_disabled_job_or_required_step_does_not_satisfy_ci_contract(self) -> None:
        mutations = (
            (
                "  documentation-sync:\n    steps:\n",
                "  documentation-sync:\n    if: false\n    steps:\n",
            ),
            (
                "      - run: python3 scripts/check_doc_sync.py\n",
                "      - run: python3 scripts/check_doc_sync.py\n        if: false\n",
            ),
        )
        for old, new in mutations:
            with self.subTest(new=new), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                create_repository(root)
                workflow = root / ".github/workflows/documentation-sync.yml"
                source = workflow.read_text(encoding="utf-8")
                self.assertIn(old, source)
                workflow.write_text(source.replace(old, new), encoding="utf-8")
                result = run_checker(root)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("ci", result.stdout.lower())

    def test_unpinned_job_level_reusable_workflow_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            workflow = root / ".github/workflows/documentation-sync.yml"
            workflow.write_text(
                workflow.read_text(encoding="utf-8")
                + "\n  reusable:\n    uses: example/example/.github/workflows/check.yml@main\n",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("commit shas", result.stdout.lower())

    def test_unpinned_remote_action_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            workflow = root / ".github/workflows/documentation-sync.yml"
            workflow.write_text(
                workflow.read_text(encoding="utf-8").replace(
                    "actions/checkout@0000000000000000000000000000000000000000",
                    "actions/checkout@v4",
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("ci", result.stdout.lower())

    def test_commands_in_yaml_comments_do_not_satisfy_ci_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            workflow = root / ".github/workflows/documentation-sync.yml"
            workflow.write_text(
                """name: Documentation Sync
on:
  push:
  pull_request:
jobs:
  documentation-sync:
    steps:
      # python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v
      # python3 scripts/check_doc_sync.py
      # cargo test --lib
      # cargo test --test router_factory
      - run: true
""",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("ci", result.stdout.lower())

    def test_duplicate_endpoint_description_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            descriptions = root / "src/endpoint_descriptions.tsv"
            descriptions.write_text(
                descriptions.read_text(encoding="utf-8")
                + "GET\t/api/v1/health\tDuplicate health\n",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("endpoint", result.stdout.lower())

    def test_system_document_version_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8").replace("v1.2.3", "v1.2.2"),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("version", result.stdout.lower())

    def test_readme_schema_count_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            readme = root / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8").replace(
                    "共 1 张应用表：", "共 2 张应用表："
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("schema", result.stdout.lower())

    def test_conflicting_readme_schema_count_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            readme = root / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8") + "\n共 2 张表：\n",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("schema", result.stdout.lower())

    def test_conflicting_schema_summary_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8")
                + "\n当前初始化 Schema 为 2 张业务及系统表。\n",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("schema", result.stdout.lower())

    def test_conflicting_schema_index_summary_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8")
                + "\n初始化 Schema 另记为 1 张应用表和 2 个应用索引。\n",
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("schema", result.stdout.lower())

    def test_schema_summary_drift_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            create_repository(root)
            system_doc = root / "docs/MarkShareX系统全貌.md"
            system_doc.write_text(
                system_doc.read_text(encoding="utf-8").replace(
                    "1 张应用表和 1 个应用索引", "2 张应用表和 1 个应用索引"
                ),
                encoding="utf-8",
            )
            result = run_checker(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("schema", result.stdout.lower())


if __name__ == "__main__":
    unittest.main()
