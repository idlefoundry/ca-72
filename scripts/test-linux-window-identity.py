#!/usr/bin/env python3
"""Check the standalone's platform dependency edges without compiling or opening it."""
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parent.parent


def window_backend(feature, target):
    result = subprocess.run(
        ["cargo", "metadata", "--offline", "--locked", "--format-version", "1",
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
