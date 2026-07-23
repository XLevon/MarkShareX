#!/usr/bin/env python3
"""Extract and boot a MarkShareX release archive for an end-to-end smoke test."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import socket
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.error
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath


ASSET_RE = re.compile(r"(?:src|href)=[\"'](/assets/[^\"']+)[\"']")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=60.0)
    return parser.parse_args()


def safe_destination(root: Path, member: str) -> Path:
    normalized = PurePosixPath(member)
    if normalized.is_absolute() or ".." in normalized.parts:
        raise ValueError(f"unsafe archive path: {member}")
    destination = (root / Path(*normalized.parts)).resolve()
    try:
        destination.relative_to(root.resolve())
    except ValueError as error:
        raise ValueError(f"unsafe archive path: {member}") from error
    return destination


def extract_archive(archive: Path, destination: Path) -> None:
    if archive.name.endswith(".tar.gz"):
        with tarfile.open(archive, "r:gz") as package:
            for member in package.getmembers():
                safe_destination(destination, member.name)
                if member.issym() or member.islnk():
                    raise ValueError(f"unsafe archive link: {member.name}")
            package.extractall(destination)
        return

    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as package:
            for member in package.infolist():
                safe_destination(destination, member.filename)
            package.extractall(destination)
        return

    raise ValueError(f"unsupported archive: {archive}")


def package_root(destination: Path) -> Path:
    roots = [path for path in destination.iterdir() if path.is_dir()]
    if len(roots) != 1:
        raise ValueError("release archive must contain exactly one root directory")
    root = roots[0]
    required = (
        root / "static/frontend/index.html",
        root / "static/frontend/assets",
        root / "config.example.toml",
    )
    for path in required:
        if not path.exists():
            raise ValueError(f"release archive is missing {path.relative_to(root)}")
    return root


def reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def sqlite_url(database: Path, *, windows: bool) -> str:
    path = database.as_posix()
    prefix = "sqlite:///" if windows else "sqlite://"
    return f"{prefix}{path}?mode=rwc"


def fetch(url: str, timeout: float = 2.0) -> tuple[int, bytes]:
    with urllib.request.urlopen(url, timeout=timeout) as response:
        return response.status, response.read()


def smoke_test(root: Path, timeout: float) -> None:
    windows = os.name == "nt"
    binary = root / ("marksharex.exe" if windows else "marksharex")
    if not binary.is_file():
        raise ValueError(f"release archive is missing {binary.name}")
    if not windows:
        binary.chmod(binary.stat().st_mode | 0o755)

    shutil.copy2(root / "config.example.toml", root / "config.toml")
    port = reserve_port()
    data_dir = root / "smoke-data"
    upload_dir = data_dir / "uploads"
    data_dir.mkdir()
    upload_dir.mkdir()

    environment = os.environ.copy()
    environment.update(
        {
            "MARKSHAREX_SERVER_HOST": "127.0.0.1",
            "MARKSHAREX_SERVER_PORT": str(port),
            "MARKSHAREX_DATA_DIR": str(data_dir),
            "MARKSHAREX_DATABASE_URL": sqlite_url(
                (data_dir / "marksharex.db").resolve(), windows=windows
            ),
            "MARKSHAREX_STORAGE_UPLOAD_DIR": str(upload_dir),
            "MARKSHAREX_AUTH_JWT_SECRET": "release-smoke-jwt-secret-not-for-production",
            "MARKSHAREX_AUTH_ENCRYPT_KEY": "release-smoke-encryption-key-32b",
            "RUST_LOG": "marksharex=info",
        }
    )

    log_path = root / "smoke-test.log"
    with log_path.open("w+", encoding="utf-8") as log:
        process = subprocess.Popen(
            [str(binary)],
            cwd=root,
            env=environment,
            stdout=log,
            stderr=subprocess.STDOUT,
            text=True,
        )
        try:
            deadline = time.monotonic() + timeout
            health_body = b""
            last_error: Exception | None = None
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    break
                try:
                    status, health_body = fetch(
                        f"http://127.0.0.1:{port}/api/v1/health"
                    )
                    if status == 200 and health_body.strip() == b"OK":
                        break
                except (OSError, urllib.error.URLError) as error:
                    last_error = error
                time.sleep(0.25)
            else:
                raise RuntimeError(f"health endpoint timed out: {last_error}")

            if process.poll() is not None:
                raise RuntimeError(f"service exited with code {process.returncode}")
            if health_body.strip() != b"OK":
                raise RuntimeError("health endpoint did not return OK")

            status, homepage = fetch(f"http://127.0.0.1:{port}/")
            if status != 200:
                raise RuntimeError(f"homepage returned HTTP {status}")
            match = ASSET_RE.search(homepage.decode("utf-8"))
            if match is None:
                raise RuntimeError("homepage does not reference a built asset")
            asset_status, asset = fetch(f"http://127.0.0.1:{port}{match.group(1)}")
            if asset_status != 200 or not asset:
                raise RuntimeError("frontend asset is not available")
        except Exception:
            log.flush()
            log.seek(0)
            print(log.read(), file=sys.stderr)
            raise
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def main() -> int:
    args = parse_args()
    try:
        if not args.archive.is_file():
            raise ValueError(f"archive does not exist: {args.archive}")
        with tempfile.TemporaryDirectory(prefix="marksharex-smoke-") as directory:
            destination = Path(directory)
            extract_archive(args.archive, destination)
            smoke_test(package_root(destination), args.timeout)
    except (OSError, RuntimeError, ValueError, zipfile.BadZipFile, tarfile.TarError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    print(f"SMOKE_TEST_PASS {args.archive.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
