//! ~/.config/latch/config: one `key=value` per line.

use crate::theme::Mode;
use gtk::glib;
use latch_core::Toggle;
use std::{fs, path::PathBuf};

pub const DEFAULT_NAME: &str = "Latch";

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub theme: Mode,
    pub name: String,
    pub icon: Option<PathBuf>,
    /// Seconds until a timed toggle locks again. 0 = off. Indexed by Toggle.
    pub autolock: [u32; 3],
    /// Unix time the lock is due. 0 = none. Indexed by Toggle.
    pub deadline: [u64; 3],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Mode::Latch,
            name: DEFAULT_NAME.into(),
            icon: None,
            autolock: [0; 3],
            deadline: [0; 3],
        }
    }
}

fn path() -> PathBuf {
    glib::user_config_dir().join("latch").join("config")
}

pub fn load() -> Config {
    parse(&fs::read_to_string(path()).unwrap_or_default())
}

pub fn update(change: impl FnOnce(&mut Config)) {
    let mut cfg = load();
    change(&mut cfg);
    let p = path();
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(p, render(&cfg));
}

fn parse(text: &str) -> Config {
    let mut cfg = Config::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim();
        match key.trim() {
            "theme" => cfg.theme = if value == "system" { Mode::System } else { Mode::Latch },
            "name" if !value.is_empty() => cfg.name = value.to_string(),
            "icon" if !value.is_empty() => cfg.icon = Some(PathBuf::from(value)),
            other => {
                for t in Toggle::ALL {
                    if other == format!("autolock_{}", t.id()) {
                        cfg.autolock[t.index()] = value.parse().unwrap_or(0);
                    } else if other == format!("deadline_{}", t.id()) {
                        cfg.deadline[t.index()] = value.parse().unwrap_or(0);
                    }
                }
            }
        }
    }
    cfg
}

fn render(cfg: &Config) -> String {
    let theme = if cfg.theme == Mode::System { "system" } else { "latch" };
    let mut out = format!("theme={theme}\nname={}\n", cfg.name);
    if let Some(icon) = &cfg.icon {
        out += &format!("icon={}\n", icon.display());
    }
    for t in Toggle::ALL.into_iter().filter(|t| t.timed()) {
        out += &format!("autolock_{}={}\n", t.id(), cfg.autolock[t.index()]);
        out += &format!("deadline_{}={}\n", t.id(), cfg.deadline[t.index()]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut cfg = Config::default();
        cfg.theme = Mode::System;
        cfg.name = "My Gate".into();
        cfg.icon = Some(PathBuf::from("/tmp/i.svg"));
        cfg.autolock[Toggle::Wine.index()] = 300;
        cfg.deadline[Toggle::Xdotool.index()] = 1_800_000_000;
        assert_eq!(parse(&render(&cfg)), cfg);
    }

    #[test]
    fn junk_and_old_format_are_safe() {
        assert_eq!(parse("theme=system\n").theme, Mode::System);
        assert_eq!(parse("garbage\nname=\n").name, DEFAULT_NAME);
        assert_eq!(parse("autolock_wine=abc").autolock[0], 0);
    }
}
