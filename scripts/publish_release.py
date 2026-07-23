#!/usr/bin/env python3
"""Publish a complete MarkShareX GitHub Release through a verified draft."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Protocol


TAG_RE = re.compile(r"^v(?P<version>[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?)$")
EXPECTED_PLATFORMS = (
    ("linux-x86_64", "tar.gz"),
    ("linux-aarch64", "tar.gz"),
    ("macos-x86_64", "tar.gz"),
    ("macos-aarch64", "tar.gz"),
    ("windows-x86_64", "zip"),
)


class ReleaseClient(Protocol):
    def get_draft_state(self, tag: str) -> bool | None: ...

    def delete_draft(self, tag: str) -> None: ...

    def create_draft(self, tag: str, assets: list[Path]) -> None: ...

    def download_assets(self, tag: str, destination: Path) -> None: ...

    def remote_asset_names(self, tag: str) -> set[str]: ...

    def publish_draft(self, tag: str) -> None: ...


def expected_asset_names(version: str) -> tuple[str, ...]:
    names: list[str] = []
    for platform, extension in EXPECTED_PLATFORMS:
        package = f"marksharex-v{version}-{platform}.{extension}"
        names.extend((package, f"{package}.sha256"))
    return tuple(names)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_assets(directory: Path, expected_names: tuple[str, ...], label: str) -> None:
    if not directory.is_dir():
        raise ValueError(f"{label} asset directory does not exist: {directory}")
    actual_names = {entry.name for entry in directory.iterdir()}
    expected_set = set(expected_names)
    if actual_names != expected_set:
        raise ValueError(
            f"unexpected {label} asset set: expected {sorted(expected_set)}, "
            f"got {sorted(actual_names)}"
        )
    if any(not (directory / name).is_file() for name in expected_names):
        raise ValueError(f"{label} asset set contains a non-file entry")

    for checksum_name in (name for name in expected_names if name.endswith(".sha256")):
        checksum_path = directory / checksum_name
        package_name = checksum_name.removesuffix(".sha256")
        lines = checksum_path.read_text(encoding="utf-8").splitlines()
        match = re.fullmatch(r"([0-9a-f]{64})  ([^/\\]+)", lines[0]) if len(lines) == 1 else None
        if match is None or match.group(2) != package_name:
            raise ValueError(f"invalid checksum file: {checksum_name}")
        actual_digest = sha256(directory / package_name)
        if actual_digest != match.group(1):
            raise ValueError(f"checksum mismatch for {package_name}")


def publish_release(tag: str, assets: Path, client: ReleaseClient) -> None:
    match = TAG_RE.fullmatch(tag)
    if match is None:
        raise ValueError(f"invalid release tag: {tag}")
    expected_names = expected_asset_names(match.group("version"))
    verify_assets(assets, expected_names, "local")

    existing_state = client.get_draft_state(tag)
    if existing_state is False:
        raise ValueError(f"Release {tag} is already published; refusing to replace it")
    if existing_state is True:
        client.delete_draft(tag)

    ordered_assets = [assets / name for name in expected_names]
    client.create_draft(tag, ordered_assets)

    with tempfile.TemporaryDirectory(prefix="marksharex-release-verify-") as directory:
        downloaded = Path(directory) / "assets"
        client.download_assets(tag, downloaded)
        verify_assets(downloaded, expected_names, "downloaded")
        for name in expected_names:
            if sha256(downloaded / name) != sha256(assets / name):
                raise ValueError(f"downloaded asset differs from local source: {name}")

    remote_names = client.remote_asset_names(tag)
    if remote_names != set(expected_names):
        raise ValueError(
            f"unexpected remote asset set: expected {sorted(expected_names)}, "
            f"got {sorted(remote_names)}"
        )

    client.publish_draft(tag)
    if client.get_draft_state(tag) is not False:
        raise ValueError(f"Release {tag} was not published successfully")


class GitHubCLIReleaseClient:
    def __init__(self, repository: str) -> None:
        self.repository = repository

    def run(self, arguments: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["gh", *arguments, "--repo", self.repository],
            check=check,
            text=True,
            capture_output=True,
        )

    def get_draft_state(self, tag: str) -> bool | None:
        result = subprocess.run(
            [
                "gh",
                "api",
                f"repos/{self.repository}/releases?per_page=100",
                "--paginate",
                "--slurp",
            ],
            check=True,
            text=True,
            capture_output=True,
        )
        pages = json.loads(result.stdout)
        if not isinstance(pages, list) or any(not isinstance(page, list) for page in pages):
            raise ValueError("unexpected GitHub Releases API response")
        releases: list[dict[str, object]] = []
        for page in pages:
            for release in page:
                if (
                    not isinstance(release, dict)
                    or not isinstance(release.get("tag_name"), str)
                    or not isinstance(release.get("draft"), bool)
                ):
                    raise ValueError("malformed GitHub Release entry")
                releases.append(release)
        matches = [release for release in releases if release["tag_name"] == tag]
        if not matches:
            return None
        if len(matches) != 1:
            raise ValueError(f"ambiguous GitHub Release state for {tag}")
        draft = matches[0]["draft"]
        if not isinstance(draft, bool):
            raise ValueError(f"ambiguous GitHub Release state for {tag}")
        return draft

    def delete_draft(self, tag: str) -> None:
        self.run(["release", "delete", tag, "--yes"])

    def create_draft(self, tag: str, assets: list[Path]) -> None:
        self.run(
            [
                "release",
                "create",
                tag,
                *(str(path) for path in assets),
                "--draft",
                "--verify-tag",
                "--title",
                f"MarkShareX {tag}",
                "--generate-notes",
                "--notes",
                (
                    "Unsigned binaries: macOS Gatekeeper or Windows SmartScreen may "
                    "require manual confirmation. Linux packages target GNU/glibc "
                    "and do not support musl."
                ),
            ]
        )

    def download_assets(self, tag: str, destination: Path) -> None:
        self.run(["release", "download", tag, "--dir", str(destination)])

    def remote_asset_names(self, tag: str) -> set[str]:
        result = self.run(
            ["release", "view", tag, "--json", "assets", "--jq", ".assets[].name"]
        )
        return {line for line in result.stdout.splitlines() if line}

    def publish_draft(self, tag: str) -> None:
        self.run(["release", "edit", tag, "--draft=false"])


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--assets", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        publish_release(
            args.tag,
            args.assets.resolve(),
            GitHubCLIReleaseClient(args.repository),
        )
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(f"RELEASE_PUBLISHED {args.tag}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
