from __future__ import annotations

import hashlib
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "package_release.py"


def write(path: Path, content: str = "content\n") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def create_inputs(root: Path, *, windows: bool = False) -> tuple[Path, Path, Path]:
    repository = root / "repository"
    frontend = root / "frontend"
    binary = root / ("marksharex.exe" if windows else "marksharex")

    write(binary, "binary\n")
    write(frontend / "index.html", "<div id=\"app\"></div>\n")
    write(frontend / "assets/index-test.js", "console.log('ok')\n")
    write(repository / "config.example.toml", "[server]\nport = 5023\n")
    write(repository / "README.md", "# MarkShareX\n")
    write(repository / "CHANGELOG.md", "# Changelog\n")
    write(repository / "LICENSE", "MIT\n")
    write(repository / "CONTRIBUTING.md", "# Contributing\n")
    write(repository / "SECURITY.md", "# Security\n")
    write(repository / "docs/CONFIG.md", "# Configuration\n")
    write(repository / "docs/MarkShareX系统全貌.md", "# System overview\n")
    write(repository / ".github/ISSUE_TEMPLATE/bug_report.yml", "name: Bug report\n")
    write(repository / ".github/ISSUE_TEMPLATE/feature_request.yml", "name: Feature request\n")
    write(repository / ".github/ISSUE_TEMPLATE/config.yml", "blank_issues_enabled: false\n")
    write(repository / ".github/pull_request_template.md", "# Pull request\n")
    return repository, frontend, binary


def run_packager(
    root: Path,
    *,
    platform: str,
    archive: str,
    windows: bool = False,
) -> subprocess.CompletedProcess[str]:
    repository, frontend, binary = create_inputs(root, windows=windows)
    return subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--repository",
            str(repository),
            "--version",
            "1.2.3",
            "--platform",
            platform,
            "--binary",
            str(binary),
            "--frontend",
            str(frontend),
            "--archive",
            archive,
            "--output-dir",
            str(root / "output"),
        ],
        text=True,
        capture_output=True,
        check=False,
    )


class PackageReleaseTests(unittest.TestCase):
    def test_linux_archive_contains_complete_runtime_and_checksum(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = run_packager(root, platform="linux-x86_64", archive="tar.gz")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

            archive = root / "output/marksharex-v1.2.3-linux-x86_64.tar.gz"
            checksum = archive.with_suffix(archive.suffix + ".sha256")
            self.assertTrue(archive.is_file())
            self.assertTrue(checksum.is_file())

            expected_digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            self.assertEqual(
                checksum.read_text(encoding="utf-8").strip(),
                f"{expected_digest}  {archive.name}",
            )

            prefix = "marksharex-v1.2.3-linux-x86_64"
            with tarfile.open(archive, "r:gz") as package:
                names = set(package.getnames())
                self.assertIn(f"{prefix}/marksharex", names)
                self.assertIn(f"{prefix}/start.sh", names)
                self.assertIn(f"{prefix}/static/frontend/index.html", names)
                self.assertIn(f"{prefix}/static/frontend/assets/index-test.js", names)
                self.assertIn(f"{prefix}/config.example.toml", names)
                self.assertIn(f"{prefix}/README.md", names)
                self.assertIn(f"{prefix}/CHANGELOG.md", names)
                self.assertIn(f"{prefix}/LICENSE", names)
                self.assertIn(f"{prefix}/CONTRIBUTING.md", names)
                self.assertIn(f"{prefix}/SECURITY.md", names)
                self.assertIn(f"{prefix}/docs/CONFIG.md", names)
                self.assertIn(f"{prefix}/docs/MarkShareX系统全貌.md", names)
                self.assertIn(f"{prefix}/.github/ISSUE_TEMPLATE/bug_report.yml", names)
                self.assertIn(f"{prefix}/.github/ISSUE_TEMPLATE/feature_request.yml", names)
                self.assertIn(f"{prefix}/.github/ISSUE_TEMPLATE/config.yml", names)
                self.assertIn(f"{prefix}/.github/pull_request_template.md", names)
                self.assertEqual(package.getmember(f"{prefix}/marksharex").mode & 0o777, 0o755)
                self.assertEqual(package.getmember(f"{prefix}/start.sh").mode & 0o777, 0o755)
                start = package.extractfile(f"{prefix}/start.sh")
                if start is None:
                    self.fail("start.sh is missing from tar archive")
                self.assertIn('cd "$(dirname "$0")"', start.read().decode("utf-8"))

    def test_windows_archive_contains_executable_and_cmd_launcher(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = run_packager(
                root,
                platform="windows-x86_64",
                archive="zip",
                windows=True,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

            archive = root / "output/marksharex-v1.2.3-windows-x86_64.zip"
            prefix = "marksharex-v1.2.3-windows-x86_64"
            with zipfile.ZipFile(archive) as package:
                names = set(package.namelist())
                self.assertIn(f"{prefix}/marksharex.exe", names)
                self.assertIn(f"{prefix}/start.cmd", names)
                self.assertIn(f"{prefix}/static/frontend/index.html", names)
                launcher = package.read(f"{prefix}/start.cmd").decode("utf-8")
                self.assertIn('cd /d "%~dp0"', launcher)
                self.assertIn("marksharex.exe", launcher)

    def test_platform_binary_and_archive_format_must_match(self) -> None:
        cases = (
            ("windows-x86_64", "tar.gz", True),
            ("windows-x86_64", "zip", False),
            ("linux-x86_64", "zip", False),
        )
        for platform, archive, windows_binary in cases:
            with self.subTest(platform=platform, archive=archive), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                result = run_packager(
                    root,
                    platform=platform,
                    archive=archive,
                    windows=windows_binary,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("platform", (result.stdout + result.stderr).lower())

    def test_missing_frontend_assets_fails_without_partial_archive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repository, frontend, binary = create_inputs(root)
            (frontend / "assets/index-test.js").unlink()
            (frontend / "assets").rmdir()
            output = root / "output"
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
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
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("assets", (result.stdout + result.stderr).lower())
            self.assertFalse(list(output.glob("*.tar.gz")) if output.exists() else [])

    def test_invalid_version_or_platform_is_rejected(self) -> None:
        for version, platform in (("../escape", "linux-x86_64"), ("1.2.3", "../linux")):
            with self.subTest(version=version, platform=platform), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                repository, frontend, binary = create_inputs(root)
                result = subprocess.run(
                    [
                        sys.executable,
                        str(SCRIPT),
                        "--repository",
                        str(repository),
                        "--version",
                        version,
                        "--platform",
                        platform,
                        "--binary",
                        str(binary),
                        "--frontend",
                        str(frontend),
                        "--archive",
                        "tar.gz",
                        "--output-dir",
                        str(root / "output"),
                    ],
                    text=True,
                    capture_output=True,
                    check=False,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("invalid", (result.stdout + result.stderr).lower())


if __name__ == "__main__":
    unittest.main()
