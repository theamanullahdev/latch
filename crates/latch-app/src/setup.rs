//! System helper: 6 root-owned copies + polkit policy. Installed by the helper itself, one admin prompt.

use latch_core::{Direction, Toggle, HELPER_DIR, HELPER_MARKER};
use std::{env, path::{Path, PathBuf}, process::Command};

fn installed_helper() -> PathBuf {
    Path::new(HELPER_DIR).join("latch-helper")
}

/// Installed copy first. Else the one next to this binary.
fn helper_bin() -> Option<PathBuf> {
    let beside = env::current_exe().ok()?.parent()?.join("latch-helper");
    [installed_helper(), beside].into_iter().find(|p| p.exists())
}

pub fn is_installed() -> bool {
    Toggle::ALL.iter().all(|t| {
        [Direction::Enable, Direction::Disable].iter().all(|d| Path::new(&t.helper_path(*d)).exists())
    })
}

/// Helper present, but not made by Latch itself: a .deb or Arch package owns it.
pub fn managed_by_package() -> bool {
    is_installed() && !Path::new(HELPER_DIR).join(HELPER_MARKER).exists()
}

pub fn install() -> Result<(), String> {
    admin("install")
}

pub fn remove() -> Result<(), String> {
    // Same refusal as the helper, but before any password prompt.
    if managed_by_package() {
        return Err("installed by a package manager. Remove it with apt, pacman or dnf instead".into());
    }
    admin("uninstall")
}

fn admin(cmd: &str) -> Result<(), String> {
    let bin = helper_bin().ok_or("latch-helper not found next to the app")?;
    let out = Command::new("pkexec").arg(&bin).arg(cmd).output().map_err(|e| e.to_string())?;
    match out.status.code() {
        Some(0) => Ok(()),
        Some(126) => Err("Cancelled".into()),
        Some(127) => Err("Not authorized".into()),
        _ => Err(String::from_utf8_lossy(&out.stderr).trim().to_string()),
    }
}
