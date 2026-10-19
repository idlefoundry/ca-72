#!/usr/bin/env python3
"""Check launcher registration and platform dependencies without opening the app."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


def window_backend(feature, target):
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1",
         "--features", f"ca72-plugin/{feature}", "--filter-platform", target],
        cwd=ROOT, capture_output=True, text=True, check=True,
    )
    metadata = json.loads(result.stdout)
    framework = next(package for package in metadata["packages"] if package["name"] == "nih_plug")
    node = next(node for node in metadata["resolve"]["nodes"] if node["id"] == framework["id"])
    backends = [dep for dep in node["deps"] if dep["name"].startswith("baseview")]
    if len(backends) != 1:
        raise AssertionError(f"Expected one standalone backend: {backends}")
    backend_id = backends[0]["pkg"]
    package = next(package for package in metadata["packages"] if package["id"] == backend_id)
    features = next(node["features"] for node in metadata["resolve"]["nodes"] if node["id"] == backend_id)
    return package, features


class WindowIdentity(unittest.TestCase):
    def test_installed_desktop_id_and_icon_agree(self):
        with tempfile.TemporaryDirectory(prefix="ca72-identity-") as temporary:
            root = Path(temporary)
            payload = root / "payload"
            payload.mkdir()
            for source in ["scripts/installer/install.sh", "scripts/standalone/CA-72.desktop",
                           "scripts/standalone/com.idlefoundry.ca72.png",
                           "THIRD-PARTY-NOTICES.txt"]:
                shutil.copy(ROOT / source, payload)
            shutil.copy(ROOT / "LICENSE", payload / "LICENSE.txt")
            (payload / "ca72-standalone").write_text("#!/bin/sh\nexit 0\n")
            prefix = root / "prefix with spaces %"
            # A custom prefix must be an existing folder (R-INSTALL-DESTINATION).
            prefix.mkdir()
            # The menu entry and icon go to the user's data folder whatever the app's prefix
            # (R-INSTALL-DESTINATION); keep both inside this test's folder.
            data = root / "data"
            env = dict(os.environ, XDG_DATA_HOME=str(data), HOME=str(root / "home"))
            subprocess.run(
                ["sh", str(payload / "install.sh"), "--standalone-only", "--prefix", str(prefix)],
                check=True, capture_output=True, text=True, env=env,
            )
            entries = list((data / "applications").glob("*.desktop"))
            self.assertEqual(len(entries), 1)
            fields = dict(line.split("=", 1) for line in entries[0].read_text().splitlines()
                          if "=" in line)
            self.assertEqual(entries[0].stem, fields["Icon"])
            self.assertEqual(fields["StartupWMClass"], "CA-72")
            icon = data / "icons/hicolor/512x512/apps" / (fields["Icon"] + ".png")
            self.assertEqual(icon.read_bytes(), (payload / (fields["Icon"] + ".png")).read_bytes())

    def test_linux_production_and_development_launchers_use_the_patched_backend(self):
        for feature in ["standalone-production", "standalone"]:
            with self.subTest(feature=feature):
                backend, features = window_backend(feature, "x86_64-unknown-linux-gnu")
                self.assertIsNone(backend["source"])
                self.assertEqual(Path(backend["manifest_path"]), ROOT / "third_party/baseview/Cargo.toml")
                # The editor shares this crate and omits its feature-gated gl_config field.
                self.assertNotIn("opengl", features)

    def test_every_platform_opens_the_patched_standalone_backend(self):
        # Since R-WIN-STANDALONE the standalone opens its window with the repository's
        # patched baseview on every platform, without OpenGL.
        for feature in ["standalone-production", "standalone"]:
            for target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc",
                           "x86_64-unknown-linux-gnu"]:
                with self.subTest(feature=feature, target=target):
                    backend, features = window_backend(feature, target)
                    self.assertNotIn("opengl", features)
                    self.assertIsNone(backend["source"])
                    self.assertTrue(backend["manifest_path"].endswith(
                        "third_party/baseview/Cargo.toml"))

if __name__ == "__main__":
    unittest.main()
