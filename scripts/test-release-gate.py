#!/usr/bin/env python3
"""A version tag's CI run is a public release: its builds and installers carry the key gate."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = (ROOT / ".github/workflows/ci.yml").read_text()
NAME = "CA72_PUBLIC_RELEASE"
# The one form the workflow uses: on for refs under the prefix, off otherwise.
GATED = re.compile(r"\$\{\{\s*startsWith\(github\.ref,\s*'([^']+)'\)\s*&&\s*'1'\s*\|\|\s*'0'\s*\}\}")


def workflow_env():
    """The workflow-level env block, which every step of every job inherits."""
    block = re.search(r"^env:\n((?:  .*\n|\s*\n)*)", WORKFLOW, re.M)
    assert block, "the workflow needs an env block"
    return dict(re.findall(r"^  ([A-Z0-9_]+): (.*)$", block.group(1), re.M))


def public_release(ref):
    gated = GATED.fullmatch(workflow_env().get(NAME, ""))
    assert gated, f"{NAME} must be set at the workflow level from the ref"
    return ref.startswith(gated.group(1))


class ReleaseGate(unittest.TestCase):
    def test_version_tags_turn_on_the_gate(self):
        self.assertTrue(public_release("refs/tags/v0.2.7"))

    def test_other_runs_build_without_it(self):
        for ref in ["refs/pull/100/merge", "refs/heads/main", "refs/heads/skin-system"]:
            with self.subTest(ref=ref):
                self.assertFalse(public_release(ref))

    def test_the_gate_matches_the_release_job(self):
        release = re.search(r"^  release:\n    name: .*\n    if: startsWith\(github\.ref, '([^']+)'\)", WORKFLOW, re.M)
        self.assertTrue(release)
        self.assertEqual(GATED.fullmatch(workflow_env()[NAME]).group(1), release.group(1))

    def test_no_job_or_step_overrides_it(self):
        # Every mention but the workflow-level one would shadow it for the bundles or installer.
        self.assertEqual(len(re.findall(rf"^\s+{NAME}:", WORKFLOW, re.M)), 1)

    def test_the_bundles_and_installer_are_built_in_ci(self):
        for step in ["cargo xtask bundle ca72-plugin --profile bundle", "run: scripts/package.sh"]:
            with self.subTest(step=step):
                self.assertIn(step, WORKFLOW)


if __name__ == "__main__":
    unittest.main()
