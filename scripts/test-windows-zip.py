"""The manual Windows distribution preserves all standalone executables and checksums."""
from pathlib import Path
import hashlib
import os
import subprocess
import tempfile
import tomllib
import unittest
import zipfile

ROOT = Path(__file__).resolve().parent.parent

class WindowsZip(unittest.TestCase):
    def test_complete_payload_and_source_provenance(self):
        with tempfile.TemporaryDirectory() as folder:
            source = Path(folder)
            files = {
                'CA-72.clap': b'plugin',
                'CA-72.vst3/Contents/x86_64-win/CA-72.vst3': b'plugin',
                'CA-72.exe': b'app', 'CA-72.ico': b'icon',
                'ca72-fx-helper.exe': b'helper', 'ca72-mcp.exe': b'bridge',
            }
            for name, data in files.items():
                p = source / name
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(data)
            env = dict(os.environ, CA72_SOURCE_COMMIT='a' * 40)
            subprocess.run(['bash', str(ROOT / 'scripts/windows-zip.sh'), str(source)], env=env, check=True, capture_output=True)
            version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
            with zipfile.ZipFile(ROOT / f'target/packages/CA-72-{version}-Windows-x86_64.zip') as archive:
                for name, data in files.items():
                    self.assertEqual(archive.read(name), data)
                self.assertIn('Source commit: ' + 'a' * 40, archive.read('README.txt').decode())
                for line in archive.read('FILE-SHA256SUMS.txt').decode().splitlines():
                    digest, name = line.split(maxsplit=1)
                    self.assertEqual(hashlib.sha256(archive.read(name)).hexdigest(), digest)
                self.assertEqual(set(archive.namelist()) - {'FILE-SHA256SUMS.txt'}, {line.split(maxsplit=1)[1] for line in archive.read('FILE-SHA256SUMS.txt').decode().splitlines()})
        workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        artifact = workflow.split('- uses: actions/upload-artifact@v7', 1)[1].split('- uses:', 1)[0]
        for name in ['ca72-fx-helper.exe', 'ca72-mcp.exe']:
            self.assertIn('target/bundled/' + name, artifact)

if __name__ == '__main__':
    unittest.main()
