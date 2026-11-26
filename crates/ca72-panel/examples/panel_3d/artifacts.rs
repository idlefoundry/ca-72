//! Shared output paths for the Cycles examples.

use std::path::{Path, PathBuf, absolute};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Command;

pub fn output(requested: Option<PathBuf>, subdir: &str) -> Result<PathBuf, String> {
    let roots = lab_roots();
    if let Some(path) = requested {
        let full = absolute(&path).map_err(|e| format!("resolve {}: {e}", path.display()))?;
        if let Some(root) = roots.iter().find(|root| full.starts_with(root))
            && !mounted(root)
        {
            return Err(format!("Lab share is not mounted at {}", root.display()));
        }
        return Ok(path);
    }
    roots
        .into_iter()
        .find(|root| mounted(root))
        .map(|root| root.join("artifacts/ca-72/panel-3d").join(subdir))
        .ok_or_else(|| "Mount the lab share or provide its artifact output path".into())
}

fn lab_roots() -> Vec<PathBuf> {
    if cfg!(target_os = "linux") {
        ["/mnt/lab", "/mnt/lab-ca72"].map(PathBuf::from).to_vec()
    } else if cfg!(target_os = "macos") {
        std::env::home_dir()
            .map(|home| home.join("lab"))
            .into_iter()
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
fn mounted(root: &Path) -> bool {
    Command::new("findmnt")
        .args(["-rnC", "-o", "FSTYPE", "-M"])
        .arg(root)
        .output()
        .is_ok_and(|out| {
            out.status.success()
                && String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .any(|kind| matches!(kind, "nfs" | "nfs4" | "cifs" | "smb3"))
        })
}

#[cfg(target_os = "macos")]
fn mounted(root: &Path) -> bool {
    Command::new("/sbin/mount").output().is_ok_and(|out| {
        out.status.success()
            && String::from_utf8_lossy(&out.stdout).lines().any(|line| {
                let Some((_, mount)) = line.split_once(" on ") else {
                    return false;
                };
                let Some((path, options)) = mount.rsplit_once(" (") else {
                    return false;
                };
                Path::new(path) == root
                    && (options.starts_with("nfs,") || options.starts_with("smbfs,"))
            })
    })
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn mounted(_root: &Path) -> bool {
    false
}
