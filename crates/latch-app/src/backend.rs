//! Run the root helper through pkexec, off the UI thread.
//! Helper missing: install it first (one admin prompt).

use gtk::gio;
use crate::setup;
use latch_core::{Direction, Toggle};
use std::process::Command;

/// pkexec exit: 126 = dismissed, 127 = not authorized.
pub async fn apply(t: Toggle, on: bool) -> Result<(), String> {
    let path = t.helper_path(Direction::from_on(on));
    gio::spawn_blocking(move || run(&path))
        .await
        .map_err(|_| "Worker crashed".to_string())?
}

/// Same job, on the calling thread. For quit-time locking.
pub fn apply_blocking(t: Toggle, on: bool) -> Result<(), String> {
    run(&t.helper_path(Direction::from_on(on)))
}

fn run(path: &str) -> Result<(), String> {
    if !setup::is_installed() {
        setup::install()?;
    }
    let out = Command::new("pkexec").arg(path).output().map_err(|e| e.to_string())?;
    match out.status.code() {
        Some(0) => Ok(()),
        Some(126) => Err("Cancelled".into()),
        Some(127) => Err("Not authorized".into()),
        _ => {
            let err = String::from_utf8_lossy(&out.stderr);
            Err(err.trim().strip_prefix("latch-helper: ").unwrap_or("Failed").to_string())
        }
    }
}
