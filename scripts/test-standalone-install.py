#!/usr/bin/env python3
"""Test Linux component installation without touching the user's profile."""
from pathlib import Path
import os
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
        self.prefix.mkdir()
        self.home = self.root / "home"
        self.data = self.home / ".local/share"
        self.environment = {**os.environ, "HOME": str(self.home), "XDG_DATA_HOME": str(self.data)}
        self.payload.mkdir()
        shutil.copy(ROOT / "scripts/installer/install.sh", self.payload / "install.sh")
        shutil.copy(ROOT / "scripts/standalone/CA-72.desktop", self.payload / "CA-72.desktop")
        for name in ["ca72-standalone", "com.idlefoundry.ca72.png", "LICENSE.txt", "THIRD-PARTY-NOTICES.txt", "CA-72.clap"]:
            (self.payload / name).write_text("fixture\n")
        (self.payload / "CA-72.vst3").mkdir()
        (self.payload / "CA-72.vst3/binary").write_text("old\n")

    def run_installer(self, *args, success=True):
        prefix = [] if "--plugins-only" in args else ["--prefix", str(self.prefix)]
        result = subprocess.run(["sh", str(self.payload / "install.sh"), *prefix, *args], env=self.environment, capture_output=True, text=True)
        self.assertEqual(result.returncode == 0, success, result.stderr)
        return result

    def test_app_only_launch_entry_and_uninstall_preserve_session(self):
        self.run_installer("--standalone-only")
        binary = self.prefix / "bin/ca72-standalone"
        self.assertTrue(binary.stat().st_mode & 0o111)
        self.assertFalse((self.prefix / "lib/clap/CA-72.clap").exists())
        self.assertFalse((self.home / ".clap/CA-72.clap").exists())
        desktop = self.data / "applications/com.idlefoundry.ca72.desktop"
        self.assertIn('sh "' + str(binary).replace("%", "%%") + '"', desktop.read_text())
        if shutil.which("desktop-file-validate"):
            subprocess.run(["desktop-file-validate", str(desktop)], check=True)
        session = self.data / "Idle Foundry/CA-72/Standalone/session.json"
        session.parent.mkdir(parents=True)
        session.write_text("saved edits")
        settings = self.data / "Idle Foundry/CA-72/settings.toml"
        settings.write_text("settings")
        preset = self.data / "Idle Foundry/CA-72/Presets/own.toml"
        preset.parent.mkdir()
        preset.write_text("my preset")
        unrelated = self.prefix / "keep.txt"
        unrelated.write_text("unrelated")
        self.run_installer("--standalone-only", "--uninstall")
        self.assertFalse(binary.exists())
        self.assertFalse(desktop.exists())
        self.assertEqual(session.read_text(), "saved edits")
        self.assertEqual(settings.read_text(), "settings")
        self.assertEqual(preset.read_text(), "my preset")
        self.assertEqual(unrelated.read_text(), "unrelated")

    def test_plugins_only_and_update_replace_whole_bundle(self):
        self.run_installer("--plugins-only")
        old = self.home / ".vst3/CA-72.vst3/stale"
        old.write_text("stale")
        self.run_installer("--plugins-only")
        self.assertFalse(old.exists())
        self.assertFalse((self.prefix / "bin/ca72-standalone").exists())

    def test_missing_selected_payload_leaves_install_untouched(self):
        self.run_installer()
        (self.payload / "ca72-standalone").unlink()
        self.run_installer(success=False)
        self.assertTrue((self.prefix / "bin/ca72-standalone").exists())
        self.assertTrue((self.home / ".clap/CA-72.clap").exists())

    def test_custom_app_keeps_plugins_and_menu_in_discovery_folders(self):
        self.run_installer()
        self.assertTrue((self.home / ".vst3/CA-72.vst3/binary").exists())
        self.assertTrue((self.home / ".clap/CA-72.clap").exists())
        self.assertFalse((self.prefix / "lib").exists())
        self.assertTrue((self.prefix / "share/Idle Foundry/CA-72/Application/LICENSE.txt").exists())
        self.assertTrue((self.data / "applications/com.idlefoundry.ca72.desktop").exists())
        (self.payload / "ca72-standalone").write_text("updated app")
        self.run_installer()
        self.assertEqual((self.prefix / "bin/ca72-standalone").read_text(), "updated app")
        self.run_installer("--uninstall")
        self.assertFalse((self.home / ".clap/CA-72.clap").exists())

    def test_missing_external_folder_leaves_plugins_untouched(self):
        self.run_installer()
        self.prefix.rename(self.root / "disconnected")
        (self.payload / "CA-72.vst3/binary").write_text("replacement")
        result = self.run_installer(success=False)
        self.assertIn("unavailable", result.stderr)
        self.assertEqual((self.home / ".vst3/CA-72.vst3/binary").read_text(), "old\n")
        self.assertFalse(self.prefix.exists())

    @unittest.skipIf(os.geteuid() == 0, "root writes through read-only permissions")
    def test_read_only_prefix_leaves_plugins_and_app_untouched(self):
        # A read-only drive passes the folder check; permissions stand in for its mount.
        self.run_installer()
        (self.payload / "CA-72.vst3/binary").write_text("replacement")
        (self.payload / "ca72-standalone").write_text("updated app")
        folders = [self.prefix, *(path for path in self.prefix.rglob("*") if path.is_dir())]
        for folder in folders:
            folder.chmod(0o555)
        self.addCleanup(lambda: [folder.chmod(0o755) for folder in folders])
        for args in [(), ("--standalone-only",)]:
            with self.subTest(args=args):
                result = self.run_installer(*args, success=False)
                self.assertIn("not writable", result.stderr)
                self.assertEqual((self.home / ".vst3/CA-72.vst3/binary").read_text(), "old\n")
                self.assertEqual((self.prefix / "bin/ca72-standalone").read_text(), "fixture\n")
        # An empty read-only folder fails the same way, before any plug-in is replaced.
        self.prefix = self.root / "empty read-only"
        self.prefix.mkdir(mode=0o555)
        self.addCleanup(self.prefix.chmod, 0o755)
        result = self.run_installer(success=False)
        self.assertIn("not writable", result.stderr)
        self.assertEqual((self.home / ".vst3/CA-72.vst3/binary").read_text(), "old\n")

    def test_prefix_must_be_absolute(self):
        self.prefix = Path("relative")
        result = self.run_installer(success=False)
        self.assertIn("must be absolute", result.stderr)

    def test_default_user_install_and_uninstall(self):
        command = ["sh", str(self.payload / "install.sh")]
        subprocess.run(command, env=self.environment, check=True, capture_output=True)
        self.assertTrue((self.home / ".local/bin/ca72-standalone").exists())
        self.assertTrue((self.home / ".vst3/CA-72.vst3/binary").exists())
        self.assertTrue((self.home / ".clap/CA-72.clap").exists())
        subprocess.run([*command, "--uninstall"], env=self.environment, check=True, capture_output=True)
        self.assertFalse((self.home / ".local/bin/ca72-standalone").exists())

    def test_prefix_with_plugins_only_is_rejected(self):
        result = subprocess.run(
            ["sh", str(self.payload / "install.sh"), "--plugins-only", "--prefix", str(self.prefix)],
            env=self.environment, capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 2)
        self.assertFalse((self.home / ".clap/CA-72.clap").exists())

    @unittest.skipUnless(shutil.which("gio"), "GIO desktop launcher is unavailable")
    def test_real_launcher_handles_paths_as_data(self):
        (self.payload / "ca72-standalone").write_text(
            '#!/usr/bin/env python3\nfrom pathlib import Path\n'
            'Path(__file__).with_name("launched").write_text("launched safely")\n'
        )
        for name in ['ordinary', 'with spaces %', 'a"quote', 'a$dollar', 'a`tick', 'a\\slash', 'a=equals']:
            with self.subTest(name=name):
                self.prefix = self.root / name
                self.prefix.mkdir()
                self.run_installer("--standalone-only")
                desktop = self.data / "applications/com.idlefoundry.ca72.desktop"
                result = subprocess.run(["gio", "launch", str(desktop)], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                marker = self.prefix / "bin/launched"
                deadline = time.monotonic() + 2.0
                while not marker.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertEqual(marker.read_text(), "launched safely")


if __name__ == "__main__":
    unittest.main()
