//! Read state without root. Wine, input tools = exec bit for others. Firewall = ufw.conf.

use crate::Toggle;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

const UFW_CONF: &str = "/etc/ufw/ufw.conf";

/// Keyboard and mouse injection tools. X11: xdotool. Wayland: ydotool, wtype, dotool.
pub const INPUT_TOOLS: [&str; 6] = ["xdotool", "ydotool", "ydotoold", "wtype", "dotool", "dotoold"];

/// Real files behind the toggle. Symlinks resolved. Missing ones dropped.
pub fn binaries(t: Toggle) -> Vec<PathBuf> {
    let names: Vec<String> = match t {
        Toggle::Wine => vec!["/usr/bin/wine".into(), "/usr/bin/wine64".into()],
        Toggle::Xdotool => INPUT_TOOLS.iter().map(|n| format!("/usr/bin/{n}")).collect(),
        Toggle::Firewall => vec![],
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for real in names.iter().filter_map(|n| fs::canonicalize(n).ok()) {
        if !found.contains(&real) {
            found.push(real);
        }
    }
    found
}

/// Some(true) = on (unlocked, enabled, active). None = not installed or unknown.
/// Several files: on if any one can still be run by others.
pub fn is_on(t: Toggle) -> Option<bool> {
    match t {
        Toggle::Firewall => parse_ufw(&fs::read_to_string(UFW_CONF).ok()?),
        _ => {
            let bins = binaries(t);
            if bins.is_empty() {
                return None;
            }
            Some(bins.iter().any(|b| fs::metadata(b).is_ok_and(|m| m.permissions().mode() & 0o001 != 0)))
        }
    }
}

pub fn parse_ufw(conf: &str) -> Option<bool> {
    conf.lines()
        .filter_map(|l| l.trim().strip_prefix("ENABLED="))
        .map(|v| v.trim().eq_ignore_ascii_case("yes"))
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_tools_cover_x11_and_wayland() {
        for tool in ["xdotool", "ydotool", "wtype"] {
            assert!(INPUT_TOOLS.contains(&tool));
        }
    }

    #[test]
    fn ufw_conf() {
        assert_eq!(parse_ufw("# x\nENABLED=yes\nLOGLEVEL=low"), Some(true));
        assert_eq!(parse_ufw("ENABLED=no"), Some(false));
        assert_eq!(parse_ufw("nothing"), None);
    }
}
