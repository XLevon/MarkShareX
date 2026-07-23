from __future__ import annotations

import hashlib
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from scripts.publish_release import (
    EXPECTED_PLATFORMS,
    GitHubCLIReleaseClient,
    publish_release,
)


VERSION = "1.2.3"
TAG = f"v{VERSION}"


def create_assets(root: Path) -> Path:
    assets = root / "assets"
    assets.mkdir()
    for platform, extension in EXPECTED_PLATFORMS:
        name = f"marksharex-v{VERSION}-{platform}.{extension}"
        content = f"package for {platform}\n".encode()
        (assets / name).write_bytes(content)
        digest = hashlib.sha256(content).hexdigest()
        (assets / f"{name}.sha256").write_text(
            f"{digest}  {name}\n", encoding="utf-8"
        )
    return assets


class FakeGitHubReleaseClient:
    def __init__(
        self,
        *,
        existing: bool | None = None,
        corrupt_download: bool = False,
        substitute_valid_download: bool = False,
        extra_remote_asset: bool = False,
    ) -> None:
        self.draft_state = existing
        self.corrupt_download = corrupt_download
        self.substitute_valid_download = substitute_valid_download
        self.extra_remote_asset = extra_remote_asset
        self.events: list[str] = []
        self.assets: list[Path] = []

    def get_draft_state(self, tag: str) -> bool | None:
        self.events.append("view")
        return self.draft_state

    def delete_draft(self, tag: str) -> None:
        self.events.append("delete")
        self.draft_state = None
        self.assets = []

    def create_draft(self, tag: str, assets: list[Path]) -> None:
        self.events.append("create")
        self.draft_state = True
        self.assets = list(assets)

    def download_assets(self, tag: str, destination: Path) -> None:
        self.events.append("download")
        destination.mkdir(parents=True, exist_ok=False)
        for source in self.assets:
            (destination / source.name).write_bytes(source.read_bytes())
        if self.corrupt_download:
            package = next(path for path in destination.iterdir() if not path.name.endswith(".sha256"))
            package.write_bytes(b"corrupted")
        if self.substitute_valid_download:
            package = next(path for path in destination.iterdir() if not path.name.endswith(".sha256"))
            content = b"different but internally valid"
            package.write_bytes(content)
            checksum = destination / f"{package.name}.sha256"
            checksum.write_text(
                f"{hashlib.sha256(content).hexdigest()}  {package.name}\n",
                encoding="utf-8",
            )

    def remote_asset_names(self, tag: str) -> set[str]:
        self.events.append("remote_names")
        names = {path.name for path in self.assets}
        if self.extra_remote_asset:
            names.add("unexpected.txt")
        return names

    def publish_draft(self, tag: str) -> None:
        self.events.append("publish")
        if self.draft_state is not True:
            raise AssertionError("attempted to publish a non-draft release")
        self.draft_state = False


class PublishReleaseTests(unittest.TestCase):
    @patch("scripts.publish_release.subprocess.run")
    def test_remote_lookup_errors_fail_closed(self, run: Mock) -> None:
        def fail_when_checked(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
            if kwargs.get("check"):
                raise subprocess.CalledProcessError(1, command, stderr="HTTP 500")
            return subprocess.CompletedProcess(command, 1, "", "HTTP 500")

        run.side_effect = fail_when_checked
        client = GitHubCLIReleaseClient("XLevon/MarkShareX")
        with self.assertRaises(subprocess.CalledProcessError):
            client.get_draft_state(TAG)

    @patch("scripts.publish_release.subprocess.run")
    def test_remote_lookup_rejects_malformed_release_entries(self, run: Mock) -> None:
        client = GitHubCLIReleaseClient("XLevon/MarkShareX")
        responses = (
            '[[{"message":"Bad credentials"}]]',
            '[["not a release"]]',
            '[[{"tag_name":1,"draft":false}]]',
            '[[{"tag_name":"other","draft":"false"}]]',
        )
        for response in responses:
            with self.subTest(response=response):
                run.return_value = subprocess.CompletedProcess([], 0, response, "")
                with self.assertRaisesRegex(ValueError, "malformed GitHub Release"):
                    client.get_draft_state(TAG)

    @patch("scripts.publish_release.subprocess.run")
    def test_remote_lookup_distinguishes_missing_draft_and_public_release(
        self, run: Mock
    ) -> None:
        client = GitHubCLIReleaseClient("XLevon/MarkShareX")
        for response, expected in (
            ("[]", None),
            ('[[{"tag_name":"v1.2.3","draft":true}]]', True),
            ('[[{"tag_name":"v1.2.3","draft":false}]]', False),
        ):
            with self.subTest(response=response):
                run.return_value = subprocess.CompletedProcess([], 0, response, "")
                self.assertIs(client.get_draft_state(TAG), expected)

    def test_success_publishes_only_after_download_and_remote_verification(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = create_assets(Path(directory))
            client = FakeGitHubReleaseClient()
            publish_release(TAG, assets, client)

        self.assertEqual(
            client.events,
            ["view", "create", "download", "remote_names", "publish", "view"],
        )
        self.assertFalse(client.draft_state)

    def test_corrupt_download_remains_draft_and_is_not_published(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = create_assets(Path(directory))
            client = FakeGitHubReleaseClient(corrupt_download=True)
            with self.assertRaisesRegex(ValueError, "checksum"):
                publish_release(TAG, assets, client)

        self.assertTrue(client.draft_state)
        self.assertNotIn("publish", client.events)

    def test_unexpected_remote_asset_remains_draft(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = create_assets(Path(directory))
            client = FakeGitHubReleaseClient(extra_remote_asset=True)
            with self.assertRaisesRegex(ValueError, "remote asset set"):
                publish_release(TAG, assets, client)

        self.assertTrue(client.draft_state)
        self.assertNotIn("publish", client.events)

    def test_downloaded_assets_must_match_local_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = create_assets(Path(directory))
            client = FakeGitHubReleaseClient(substitute_valid_download=True)
            with self.assertRaisesRegex(ValueError, "differs from local source"):
                publish_release(TAG, assets, client)

        self.assertTrue(client.draft_state)
        self.assertNotIn("publish", client.events)

    def test_existing_public_release_is_never_replaced(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = create_assets(Path(directory))
            client = FakeGitHubReleaseClient(existing=False)
            with self.assertRaisesRegex(ValueError, "already published"):
                publish_release(TAG, assets, client)

        self.assertEqual(client.events, ["view"])

    def test_existing_draft_is_deleted_before_clean_rebuild(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = create_assets(Path(directory))
            client = FakeGitHubReleaseClient(existing=True)
            publish_release(TAG, assets, client)

        self.assertEqual(client.events[:3], ["view", "delete", "create"])
        self.assertFalse(client.draft_state)

    def test_local_asset_set_must_be_exact_before_github_is_called(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            assets = create_assets(Path(directory))
            (assets / "unexpected.txt").write_text("unexpected", encoding="utf-8")
            client = FakeGitHubReleaseClient()
            with self.assertRaisesRegex(ValueError, "local asset set"):
                publish_release(TAG, assets, client)

        self.assertEqual(client.events, [])


if __name__ == "__main__":
    unittest.main()
