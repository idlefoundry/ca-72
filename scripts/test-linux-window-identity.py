#!/usr/bin/env python3
"""Check launcher registration and platform dependencies without opening the app."""
import json
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
            subprocess.run(
                ["sh", str(payload / "install.sh"), "--standalone-only", "--prefix", str(prefix)],
                check=True, capture_output=True, text=True,
            )
            entries = list((prefix / "share/applications").glob("*.desktop"))
            self.assertEqual(len(entries), 1)
            fields = dict(line.split("=", 1) for line in entries[0].read_text().splitlines()
                          if "=" in line)
            self.assertEqual(entries[0].stem, fields["Icon"])
            self.assertEqual(fields["StartupWMClass"], "CA-72")
            icon = prefix / "share/icons/hicolor/512x512/apps" / (fields["Icon"] + ".png")
            self.assertEqual(icon.read_bytes(), (payload / (fields["Icon"] + ".png")).read_bytes())

    def test_linux_production_and_development_launchers_use_the_patched_backend(self):
        for feature in ["standalone-production", "standalone"]:
            with self.subTest(feature=feature):
                backend, features = window_backend(feature, "x86_64-unknown-linux-gnu")
                self.assertIsNone(backend["source"])
                self.assertEqual(Path(backend["manifest_path"]), ROOT / "third_party/baseview/Cargo.toml")
                # The editor shares this crate and omits its feature-gated gl_config field.
                self.assertNotIn("opengl", features)

    def test_other_platforms_keep_the_existing_standalone_backend(self):
        for feature in ["standalone-production", "standalone"]:
            for target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"]:
                with self.subTest(feature=feature, target=target):
                    backend, features = window_backend(feature, target)
                    self.assertIn("opengl", features)
                    self.assertEqual(
                        backend["source"],
                        "git+https://github.com/RustAudio/baseview.git?rev="
                        "579130ecb4f9f315ae52190af42f0ea46aeaa4a2#579130ecb4f9f315ae52190af42f0ea46aeaa4a2",
                    )


if __name__ == "__main__":
    unittest.main()
