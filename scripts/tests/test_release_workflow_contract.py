from __future__ import annotations

import re
import unittest
from pathlib import Path


WORKFLOW = Path(__file__).resolve().parents[2] / ".github/workflows/release.yml"
EXPECTED_CONDITION = (
    "github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')"
)
EXPECTED_PUBLISH_JOB = """    name: Publish GitHub Release
    if: github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')
    needs: build-binaries
    runs-on: ubuntu-22.04
    timeout-minutes: 20
    permissions:
      contents: write

    steps:
      - name: Check out repository
        uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
        with:
          fetch-depth: 0

      - name: Set up Python
        uses: actions/setup-python@a26af69be951a213d495a4c3e4e4022e16d87065 # v5
        with:
          python-version: "3.11"

      - name: Download all platform packages
        uses: actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093 # v4
        with:
          pattern: release-*
          path: release-assets
          merge-multiple: true

      - name: Publish verified GitHub Release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          python scripts/publish_release.py \\
            --repository "$GITHUB_REPOSITORY" \\
            --tag "$GITHUB_REF_NAME" \\
            --assets release-assets
"""


def validate_workflow_boundary(workflow: str) -> list[str]:
    errors: list[str] = []
    marker = "\n  publish:\n"
    if workflow.count(marker) != 1:
        return ["publish job must exist exactly once"]
    publish_job = workflow.split(marker, maxsplit=1)[1]

    condition = re.search(r"^    if: (.+)$", publish_job, re.MULTILINE)
    if condition is None or condition.group(1) != EXPECTED_CONDITION:
        errors.append("publish job must require a tag push event")
    semantic_lines = [line.split("#", maxsplit=1)[0].rstrip() for line in workflow.splitlines()]
    write_permissions = [line for line in semantic_lines if line.strip() == "contents: write"]
    if write_permissions != ["      contents: write"]:
        errors.append("only the publish job may receive contents write permission")
    if publish_job != EXPECTED_PUBLISH_JOB:
        errors.append("the entire write-permission publish job must match the audited whitelist")
    return errors


class ReleaseWorkflowContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_current_workflow_satisfies_boundary_contract(self) -> None:
        self.assertEqual(validate_workflow_boundary(self.workflow), [])

    def test_manual_dispatch_publish_mutation_is_rejected(self) -> None:
        mutated = self.workflow.replace(
            EXPECTED_CONDITION,
            "github.event_name == 'workflow_dispatch' || startsWith(github.ref, 'refs/tags/v')",
            1,
        )
        self.assertTrue(validate_workflow_boundary(mutated))

    def test_inline_or_duplicate_publisher_is_rejected(self) -> None:
        for insertion in (
            "\n          gh release edit v1.2.3 --draft=false",
            "\n          python scripts/publish_release.py",
            "\n    permissions:\n      contents: write",
        ):
            with self.subTest(insertion=insertion):
                self.assertTrue(validate_workflow_boundary(self.workflow + insertion))

    def test_disabled_or_extended_publisher_step_is_rejected(self) -> None:
        for old, new in (
            (
                "          python scripts/publish_release.py \\",
                "          if false; then python scripts/publish_release.py \\",
            ),
            (
                "            --assets release-assets\n",
                "            --assets release-assets\n          gh api /user\n",
            ),
        ):
            with self.subTest(old=old):
                mutated = self.workflow.replace(old, new, 1)
                self.assertTrue(validate_workflow_boundary(mutated))
    def test_preceding_alternative_publish_paths_are_rejected(self) -> None:
        marker = "      - name: Publish verified GitHub Release\n"
        mutations = (
            """      - name: Bypass draft verification
        run: gh api repos/example/releases/1 -X PATCH -F draft=false

""",
            """      - name: Rewrite audited publisher
        run: python -c 'open("scripts/publish_release.py", "w").write("pass")'

""",
        )
        for insertion in mutations:
            with self.subTest(insertion=insertion):
                mutated = self.workflow.replace(marker, insertion + marker, 1)
                self.assertTrue(validate_workflow_boundary(mutated))

    def test_permission_comment_cannot_impersonate_write_permission(self) -> None:
        mutated = self.workflow.replace(
            "      contents: write",
            "      contents: read # contents: write",
            1,
        )
        self.assertTrue(validate_workflow_boundary(mutated))


if __name__ == "__main__":
    unittest.main()
