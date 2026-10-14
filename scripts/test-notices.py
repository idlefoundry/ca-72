#!/usr/bin/env python3
"""Different sources of the same crate retain every licence and provenance."""

import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("notices", Path(__file__).with_name("notices.py"))
notices = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notices)


class NoticeSources(unittest.TestCase):
    def test_colliding_crates_retain_both_attributions_in_any_enumeration_order(self):
        with tempfile.TemporaryDirectory() as directory:
            packages = []
            for label, source in [
                ("Vendored Fixture Contributors", None),
                ("Pinned Fixture Contributors", "git+https://example.invalid/fixture?rev=" + "1" * 40 + "#" + "1" * 40),
            ]:
                folder = Path(directory) / label
                folder.mkdir()
                (folder / "LICENSE-MIT").write_text(notices.MIT.format(authors=label))
                packages.append({
                    "name": "same-crate", "version": "1.0.0", "source": source,
                    "repository": "https://example.invalid/fixture",
                    "manifest_path": str(folder / "Cargo.toml"), "license": "MIT",
                })
            outputs = []
            for order in [packages, list(reversed(packages))]:
                with patch.object(notices, "metadata", return_value=None), \
                     patch.object(notices, "linked", return_value=order):
                    outputs.append(notices.render())
            self.assertEqual(outputs[0], outputs[1])
            self.assertIn("Copyright (c) Vendored Fixture Contributors", outputs[0])
            self.assertIn("Copyright (c) Pinned Fixture Contributors", outputs[0])
            self.assertIn("https://example.invalid/fixture at 111111111111", outputs[0])


if __name__ == "__main__":
    unittest.main()
