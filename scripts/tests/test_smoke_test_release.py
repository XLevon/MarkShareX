from __future__ import annotations

import io
import subprocess
import sys
import tarfile
import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.smoke_test_release import sqlite_url


PACKAGER = Path(__file__).resolve().parents[1] / "package_release.py"
SMOKE_TEST = Path(__file__).resolve().parents[1] / "smoke_test_release.py"


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")


def create_test_archive(root: Path) -> Path:
    repository = root / "repository"
    frontend = root / "frontend"
    binary = root / "marksharex"
    write(repository / "config.example.toml", "[server]\nport = 5023\n")
    for name in ("README.md", "CHANGELOG.md", "LICENSE", "CONTRIBUTING.md", "SECURITY.md"):
        write(repository / name, f"{name}\n")
    write(repository / "docs/CONFIG.md", "# Configuration\n")
    write(repository / "docs/MarkShareX系统全貌.md", "# System overview\n")
    write(repository / ".github/ISSUE_TEMPLATE/bug_report.yml", "name: Bug report\n")
    write(repository / ".github/ISSUE_TEMPLATE/feature_request.yml", "name: Feature request\n")
    write(repository / ".github/ISSUE_TEMPLATE/config.yml", "blank_issues_enabled: false\n")
    write(repository / ".github/pull_request_template.md", "# Pull request\n")
    write(
        frontend / "index.html",
        '<div id="app"></div><script src="/assets/index-test.js"></script>\n',
    )
    write(frontend / "assets/index-test.js", "console.log('ok')\n")
    write(
        binary,
        r"""
        #!/usr/bin/env python3
        import http.server
        import os
        from pathlib import Path

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == "/api/v1/health":
                    body = b"OK"
                    content_type = "text/plain"
                elif self.path == "/api/v1/openapi.json":
                    body = b'{"info":{"version":"1.2.3"}}'
                    content_type = "application/json"
                elif self.path == "/":
                    body = Path("static/frontend/index.html").read_bytes()
                    content_type = "text/html"
                elif self.path == "/assets/index-test.js":
                    body = Path("static/frontend/assets/index-test.js").read_bytes()
                    content_type = "application/javascript"
                else:
                    self.send_error(404)
                    return
                self.send_response(200)
                self.send_header("Content-Type", content_type)
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, format, *args):
                pass

        port = int(os.environ["MARKSHAREX_SERVER_PORT"])
        http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
        """,
    )
    binary.chmod(0o755)
    output = root / "output"
    result = subprocess.run(
        [
            sys.executable,
            str(PACKAGER),
            "--repository",
            str(repository),
            "--version",
            "1.2.3",
            "--platform",
            "linux-x86_64",
            "--binary",
            str(binary),
            "--frontend",
            str(frontend),
            "--archive",
            "tar.gz",
            "--output-dir",
            str(output),
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise AssertionError(result.stdout + result.stderr)
    return output / "marksharex-v1.2.3-linux-x86_64.tar.gz"


class SmokeTestReleaseTests(unittest.TestCase):
    def test_sqlite_url_uses_windows_drive_url_form(self) -> None:
        self.assertEqual(
            sqlite_url(Path("C:/release/smoke-data/marksharex.db"), windows=True),
            "sqlite:///C:/release/smoke-data/marksharex.db?mode=rwc",
        )

    def test_complete_archive_starts_and_serves_frontend_asset(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = create_test_archive(Path(directory))
            result = subprocess.run(
                [
                    sys.executable,
                    str(SMOKE_TEST),
                    "--archive",
                    str(archive),
                    "--expected-version",
                    "1.2.3",
                ],
                text=True,
                capture_output=True,
                check=False,
                timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("SMOKE_TEST_PASS", result.stdout)

    def test_archive_with_wrong_binary_version_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = create_test_archive(Path(directory))
            result = subprocess.run(
                [
                    sys.executable,
                    str(SMOKE_TEST),
                    "--archive",
                    str(archive),
                    "--expected-version",
                    "9.9.9",
                ],
                text=True,
                capture_output=True,
                check=False,
                timeout=30,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("version", (result.stdout + result.stderr).lower())

    def test_archive_path_traversal_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "malicious.tar.gz"
            with tarfile.open(archive, "w:gz") as package:
                payload = b"escape\n"
                info = tarfile.TarInfo("../escape")
                info.size = len(payload)
                package.addfile(info, io.BytesIO(payload))

            result = subprocess.run(
                [
                    sys.executable,
                    str(SMOKE_TEST),
                    "--archive",
                    str(archive),
                    "--expected-version",
                    "1.2.3",
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unsafe", (result.stdout + result.stderr).lower())
            self.assertFalse((root / "escape").exists())

    def test_archive_special_file_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "special.tar.gz"
            with tarfile.open(archive, "w:gz") as package:
                directory_info = tarfile.TarInfo("package")
                directory_info.type = tarfile.DIRTYPE
                package.addfile(directory_info)
                fifo = tarfile.TarInfo("package/fifo")
                fifo.type = tarfile.FIFOTYPE
                package.addfile(fifo)

            result = subprocess.run(
                [
                    sys.executable,
                    str(SMOKE_TEST),
                    "--archive",
                    str(archive),
                    "--expected-version",
                    "1.2.3",
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("special", (result.stdout + result.stderr).lower())


if __name__ == "__main__":
    unittest.main()
