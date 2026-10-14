#!/usr/bin/env python3
"""Test Linux component installation without touching the user's profile."""
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parent.parent


class Installation(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="ca72-install-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.payload = self.root / "payload"
        self.prefix = self.root / "prefix with spaces %"
        self.payload.mkdir()
        shutil.copy(ROOT / "scripts/installer/install.sh", self.payload / "install.sh")
        shutil.copy(ROOT / "scripts/standalone/CA-72.desktop", self.payload / "CA-72.desktop")
        for name in ["ca72-standalone", "com.idlefoundry.ca72.png", "LICENSE.txt", "THIRD-PARTY-NOTICES.txt", "CA-72.clap"]:
            (self.payload / name).write_text("fixture\n")
        (self.payload / "CA-72.vst3").mkdir()
        (self.payload / "CA-72.vst3/binary").write_text("old\n")

    def run_installer(self, *args, success=True):
        result = subprocess.run(["sh", str(self.payload / "install.sh"), "--prefix", str(self.prefix), *args], capture_output=True, text=True)
        self.assertEqual(result.returncode == 0, success, result.stderr)
        return result

    def test_app_only_launch_entry_and_uninstall_preserve_session(self):
        self.run_installer("--standalone-only")
        binary = self.prefix / "bin/ca72-standalone"
        self.assertTrue(binary.stat().st_mode & 0o111)
        self.assertFalse((self.prefix / "lib/clap/CA-72.clap").exists())
        desktop = self.prefix / "share/applications/com.idlefoundry.ca72.desktop"
        self.assertIn('sh "' + str(binary).replace("%", "%%") + '"', desktop.read_text())
        if shutil.which("desktop-file-validate"):
            subprocess.run(["desktop-file-validate", str(desktop)], check=True)
        session = self.prefix / "share/Idle Foundry/CA-72/session.json"
        session.write_text("saved edits")
        self.run_installer("--standalone-only", "--uninstall")
        self.assertFalse(binary.exists())
        self.assertFalse(desktop.exists())
        self.assertEqual(session.read_text(), "saved edits")

    def test_plugins_only_and_update_replace_whole_bundle(self):
        self.run_installer("--plugins-only")
        old = self.prefix / "lib/vst3/CA-72.vst3/stale"
        old.write_text("stale")
        self.run_installer("--plugins-only")
        self.assertFalse(old.exists())
        self.assertFalse((self.prefix / "bin/ca72-standalone").exists())

    def test_missing_selected_payload_leaves_install_untouched(self):
        self.run_installer()
        (self.payload / "ca72-standalone").unlink()
        self.run_installer(success=False)
        self.assertTrue((self.prefix / "bin/ca72-standalone").exists())
        self.assertTrue((self.prefix / "lib/clap/CA-72.clap").exists())

    @unittest.skipUnless(shutil.which("gio"), "GIO desktop launcher is unavailable")
    def test_real_launcher_handles_paths_as_data(self):
        (self.payload / "ca72-standalone").write_text(
            '#!/usr/bin/env python3\nfrom pathlib import Path\n'
            'Path(__file__).with_name("launched").write_text("launched safely")\n'
        )
        for name in ['ordinary', 'with spaces %', 'a"quote', 'a$dollar', 'a`tick', 'a\\slash', 'a=equals']:
            with self.subTest(name=name):
                self.prefix = self.root / name
                self.run_installer("--standalone-only")
                desktop = self.prefix / "share/applications/com.idlefoundry.ca72.desktop"
                result = subprocess.run(["gio", "launch", str(desktop)], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                marker = self.prefix / "bin/launched"
                deadline = time.monotonic() + 2.0
                while not marker.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertEqual(marker.read_text(), "launched safely")


if __name__ == "__main__":
    unittest.main()
