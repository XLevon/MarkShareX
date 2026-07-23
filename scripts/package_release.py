#!/usr/bin/env python3
"""Build a complete, checksummed MarkShareX release archive."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import tarfile
import tempfile
import zipfile
from pathlib import Path


VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$")
PLATFORM_RE = re.compile(r"^[a-z0-9][a-z0-9_-]*$")
REQUIRED_DOCUMENTS = ("README.md", "CHANGELOG.md", "LICENSE")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path.cwd())
    parser.add_argument("--version", required=True)
    parser.add_argument("--platform", required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--frontend", type=Path, required=True)
    parser.add_argument("--archive", choices=("tar.gz", "zip"), required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    return parser.parse_args()


def require_file(path: Path, label: str) -> None:
    if not path.is_file():
        raise ValueError(f"missing {label}: {path}")


def validate_inputs(args: argparse.Namespace) -> None:
    if not VERSION_RE.fullmatch(args.version):
        raise ValueError(f"invalid version: {args.version}")
    if not PLATFORM_RE.fullmatch(args.platform):
        raise ValueError(f"invalid platform: {args.platform}")

    windows_platform = args.platform.startswith("windows-")
    windows_binary = args.binary.name.lower().endswith(".exe")
    expected_archive = "zip" if windows_platform else "tar.gz"
    if windows_platform != windows_binary or args.archive != expected_archive:
        raise ValueError(
            f"platform {args.platform} requires "
            f"{'an .exe and zip' if windows_platform else 'a non-.exe binary and tar.gz'}"
        )

    require_file(args.binary, "binary")
    require_file(args.frontend / "index.html", "frontend index.html")
    assets = args.frontend / "assets"
    if not assets.is_dir() or not any(path.is_file() for path in assets.rglob("*")):
        raise ValueError(f"missing frontend assets: {assets}")

    require_file(args.repository / "config.example.toml", "config.example.toml")
    for name in REQUIRED_DOCUMENTS:
        require_file(args.repository / name, name)


def write_launcher(package: Path, windows: bool) -> None:
    if windows:
        launcher = package / "start.cmd"
        launcher.write_text(
            '@echo off\r\ncd /d "%~dp0"\r\nmarksharex.exe\r\n',
            encoding="utf-8",
        )
    else:
        launcher = package / "start.sh"
        launcher.write_text(
            '#!/usr/bin/env bash\nset -euo pipefail\n\ncd "$(dirname "$0")"\nexec ./marksharex\n',
            encoding="utf-8",
        )
        launcher.chmod(0o755)


def create_package_tree(args: argparse.Namespace, package: Path) -> None:
    windows = args.binary.name.lower().endswith(".exe")
    binary_name = "marksharex.exe" if windows else "marksharex"
    shutil.copy2(args.binary, package / binary_name)
    if not windows:
        (package / binary_name).chmod(0o755)

    shutil.copytree(args.frontend, package / "static/frontend")
    shutil.copy2(args.repository / "config.example.toml", package / "config.example.toml")
    for name in REQUIRED_DOCUMENTS:
        shutil.copy2(args.repository / name, package / name)
    write_launcher(package, windows)


def normalize_tar_mode(info: tarfile.TarInfo) -> tarfile.TarInfo:
    if info.isdir():
        info.mode = 0o755
    elif info.name.endswith("/marksharex") or info.name.endswith("/start.sh"):
        info.mode = 0o755
    else:
        info.mode = 0o644
    info.uid = 0
    info.gid = 0
    info.uname = "root"
    info.gname = "root"
    return info


def create_archive(package: Path, destination: Path, kind: str) -> None:
    if kind == "tar.gz":
        with tarfile.open(destination, "w:gz", format=tarfile.PAX_FORMAT) as archive:
            archive.add(package, arcname=package.name, filter=normalize_tar_mode)
        return

    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(package.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(package.parent).as_posix())


def write_checksum(archive: Path) -> Path:
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    checksum = archive.with_suffix(archive.suffix + ".sha256")
    checksum.write_text(f"{digest}  {archive.name}\n", encoding="utf-8")
    return checksum


def package_release(args: argparse.Namespace) -> tuple[Path, Path]:
    validate_inputs(args)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    package_name = f"marksharex-v{args.version}-{args.platform}"
    extension = ".tar.gz" if args.archive == "tar.gz" else ".zip"
    destination = args.output_dir / f"{package_name}{extension}"

    with tempfile.TemporaryDirectory(prefix="marksharex-release-") as directory:
        package = Path(directory) / package_name
        package.mkdir()
        create_package_tree(args, package)
        temporary_archive = Path(directory) / destination.name
        create_archive(package, temporary_archive, args.archive)
        os.replace(temporary_archive, destination)

    checksum = write_checksum(destination)
    return destination, checksum


def main() -> int:
    try:
        archive, checksum = package_release(parse_args())
    except (OSError, ValueError) as error:
        print(f"error: {error}", file=__import__("sys").stderr)
        return 1

    print(archive)
    print(checksum)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
