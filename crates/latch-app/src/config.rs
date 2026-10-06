//! ~/.config/latch/config, one `theme=latch|system` line.

use crate::theme::Mode;
use std::{fs, path::PathBuf};

fn path() -> Option<PathBuf> {
    Some(gtk::glib::user_config_dir().join("latch").join("config"))
}

pub fn load_theme() -> Mode {
    let text = path().and_then(|p| fs::read_to_string(p).ok()).unwrap_or_default();
    match text.trim() {
        "theme=system" => Mode::System,
        _ => Mode::Latch,
    }
}

pub fn save_theme(mode: Mode) {
    let Some(p) = path() else { return };
    let line = if mode == Mode::System { "theme=system\n" } else { "theme=latch\n" };
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(p, line);
}
