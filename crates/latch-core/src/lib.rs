//! Toggles, password rules, state probe. No gtk.

pub mod probe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toggle {
    Wine,
    Xdotool,
    Firewall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Enable,
    Disable,
}

impl Direction {
    pub fn id(self) -> &'static str {
        match self {
            Direction::Enable => "enable",
            Direction::Disable => "disable",
        }
    }

    pub fn from_on(on: bool) -> Self {
        if on { Direction::Enable } else { Direction::Disable }
    }
}

pub const HELPER_DIR: &str = "/usr/libexec/latch";

impl Toggle {
    pub const ALL: [Toggle; 3] = [Toggle::Wine, Toggle::Xdotool, Toggle::Firewall];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn id(self) -> &'static str {
        match self {
            Toggle::Wine => "wine",
            Toggle::Xdotool => "xdotool",
            Toggle::Firewall => "firewall",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Toggle::Wine => "Wine",
            Toggle::Xdotool => "xdotool",
            Toggle::Firewall => "Firewall",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Toggle::Wine => "Run .exe files through Wine.",
            Toggle::Xdotool => "Let agents control keyboard and mouse.",
            Toggle::Firewall => "Block incoming connections (ufw).",
        }
    }

    pub fn state_text(self, on: bool) -> &'static str {
        match (self, on) {
            (Toggle::Wine, true) => "Unlocked",
            (Toggle::Wine, false) => "Locked",
            (Toggle::Xdotool, true) => "Enabled",
            (Toggle::Xdotool, false) => "Disabled",
            (Toggle::Firewall, true) => "Active",
            (Toggle::Firewall, false) => "Off",
        }
    }

    /// Path of the root helper for one action. polkit gates by this path.
    pub fn helper_path(self, dir: Direction) -> String {
        format!("{HELPER_DIR}/{}-{}", self.id(), dir.id())
    }

    pub fn from_id(id: &str) -> Option<Toggle> {
        Toggle::ALL.into_iter().find(|t| t.id() == id)
    }

    /// Risky = exposed. Firewall is risky when off.
    pub fn is_risky(self, on: bool) -> bool {
        match self {
            Toggle::Firewall => !on,
            _ => on,
        }
    }

    /// Going riskier asks password. Going safer is free.
    pub fn needs_password(self, dir: Direction) -> bool {
        let on = dir == Direction::Enable;
        self.is_risky(on)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_path_roundtrip() {
        let p = Toggle::Wine.helper_path(Direction::Enable);
        assert_eq!(p, "/usr/libexec/latch/wine-enable");
        assert_eq!(Toggle::from_id("firewall"), Some(Toggle::Firewall));
        assert_eq!(Toggle::from_id("nope"), None);
    }

    #[test]
    fn riskier_asks_safer_free() {
        assert!(Toggle::Wine.needs_password(Direction::Enable));
        assert!(!Toggle::Wine.needs_password(Direction::Disable));
        assert!(Toggle::Xdotool.needs_password(Direction::Enable));
        assert!(!Toggle::Xdotool.needs_password(Direction::Disable));
        assert!(Toggle::Firewall.needs_password(Direction::Disable));
        assert!(!Toggle::Firewall.needs_password(Direction::Enable));
    }
}
