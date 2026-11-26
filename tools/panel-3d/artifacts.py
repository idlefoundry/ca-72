"""Output paths shared by the Cycles tools."""

import os
import subprocess
import sys
from pathlib import Path


def _lab_roots() -> tuple[Path, ...]:
    if sys.platform == "linux":
        return Path("/mnt/lab"), Path("/mnt/lab-ca72")
    if sys.platform == "darwin":
        return (Path.home() / "lab",)
    return ()


def _mounted(root: Path) -> bool:
    try:
        if sys.platform == "linux":
            out = subprocess.run(
                ["findmnt", "-rnC", "-o", "FSTYPE", "-M", root],
                capture_output=True, text=True, check=False,
            )
            return out.returncode == 0 and bool(
                {"nfs", "nfs4", "cifs", "smb3"}.intersection(out.stdout.splitlines())
            )
        if sys.platform == "darwin":
            out = subprocess.run(["/sbin/mount"], capture_output=True, text=True, check=False)
            return out.returncode == 0 and any(
                line.partition(" on ")[2].startswith((f"{root} (nfs,", f"{root} (smbfs,"))
                for line in out.stdout.splitlines()
            )
    except OSError:
        return False
    return False


def output_dir(requested: Path | None = None, subdir: str = "") -> Path:
    roots = _lab_roots()
    if requested is not None:
        full = Path(os.path.abspath(requested))
        for root in roots:
            if full.is_relative_to(root) and not _mounted(root):
                raise SystemExit(f"Lab share is not mounted at {root}")
        return requested
    for root in roots:
        if _mounted(root):
            return root / "artifacts" / "ca-72" / "panel-3d" / subdir
    raise SystemExit("Mount the lab share or provide its artifact output path")
