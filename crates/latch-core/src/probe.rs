//! Read state without root. Wine, xdotool = exec bit for others. Firewall = ufw.conf.

use crate::Toggle;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

const UFW_CONF: &str = "/etc/ufw/ufw.conf";

/// Real files behind the toggle. Symlinks resolved. Missing ones dropped.
pub fn binaries(t: Toggle) -> Vec<PathBuf> {
    let names: &[&str] = match t {
        Toggle::Wine => &["/usr/bin/wine", "/usr/bin/wine64"],
        Toggle::Xdotool => &["/usr/bin/xdotool"],
        Toggle::Firewall => &[],
    };
    let mut found: Vec<PathBuf> = names.iter().filter_map(|n| fs::canonicalize(n).ok()).collect();
    found.dedup();
    found
}

/// Some(true) = on (unlocked, enabled, active). None = not installed or unknown.
pub fn is_on(t: Toggle) -> Option<bool> {
    match t {
        Toggle::Firewall => parse_ufw(&fs::read_to_string(UFW_CONF).ok()?),
        _ => {
            let bin = binaries(t).into_iter().next()?;
            Some(fs::metadata(bin).ok()?.permissions().mode() & 0o001 != 0)
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
    fn ufw_conf() {
        assert_eq!(parse_ufw("# x\nENABLED=yes\nLOGLEVEL=low"), Some(true));
        assert_eq!(parse_ufw("ENABLED=no"), Some(false));
        assert_eq!(parse_ufw("nothing"), None);
    }
}
