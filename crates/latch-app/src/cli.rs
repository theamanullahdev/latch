//! Terminal use, no window. For scripts and status bars (Waybar on Hyprland).
//!   latch status [--json]      show state. --json = Waybar custom module format
//!   latch set <toggle> <on|off>  flip one switch (polkit asks if it needs a password)

use crate::backend;
use latch_core::{probe, Toggle};

const HELP: &str = "\
latch                       open the window (or: latch wine | xdotool | firewall | settings)
latch status [--json]       show the state. --json prints Waybar custom module JSON
latch set <toggle> <on|off> flip one switch. toggles: wine, xdotool, firewall
latch help                  this text";

/// Some(exit code) = handled here, no window. None = open the window.
pub fn run(args: &[String]) -> Option<i32> {
    match args.first().map(String::as_str) {
        Some("status") => {
            let json = args.iter().any(|a| a == "--json");
            println!("{}", if json { waybar(&states()) } else { plain(&states()) });
            Some(0)
        }
        Some("set") => Some(set(&args[1..])),
        Some("help" | "-h" | "--help") => {
            println!("{HELP}");
            Some(0)
        }
        _ => None,
    }
}

type States = Vec<(Toggle, Option<bool>)>;

fn states() -> States {
    Toggle::ALL.iter().map(|t| (*t, probe::is_on(*t))).collect()
}

fn state_label(t: Toggle, on: Option<bool>) -> &'static str {
    on.map_or("not found", |on| t.state_text(on))
}

fn exposed(states: &States) -> usize {
    states.iter().filter(|(t, on)| on.is_some_and(|on| t.is_risky(on))).count()
}

fn plain(states: &States) -> String {
    states
        .iter()
        .map(|(t, on)| format!("{:<9} {}", t.name(), state_label(*t, *on)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// {"text": ..., "class": "safe" | "exposed", "tooltip": ...}
fn waybar(states: &States) -> String {
    let n = exposed(states);
    let (text, class) = if n == 0 { ("All safe".to_string(), "safe") } else { (format!("{n} exposed"), "exposed") };
    let tooltip = states
        .iter()
        .map(|(t, on)| format!("{}: {}", t.name(), state_label(*t, *on)))
        .collect::<Vec<_>>()
        .join("\\n");
    format!("{{\"text\":\"{text}\",\"class\":\"{class}\",\"tooltip\":\"{tooltip}\"}}")
}

fn parse_set(args: &[String]) -> Result<(Toggle, bool), String> {
    let [name, state] = args else {
        return Err("usage: latch set <wine|xdotool|firewall> <on|off>".into());
    };
    let toggle = Toggle::from_id(name).ok_or(format!("unknown toggle '{name}'"))?;
    match state.as_str() {
        "on" => Ok((toggle, true)),
        "off" => Ok((toggle, false)),
        other => Err(format!("expected on or off, got '{other}'")),
    }
}

fn set(args: &[String]) -> i32 {
    let (toggle, on) = match parse_set(args) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("latch: {e}");
            return 2;
        }
    };
    if let Err(e) = backend::apply_blocking(toggle, on) {
        eprintln!("latch: {e}");
        return 1;
    }
    let now = probe::is_on(toggle);
    println!("{} {}", toggle.name(), state_label(toggle, now));
    if now == Some(on) { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(list: &[&str]) -> Vec<String> {
        list.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn waybar_all_safe_and_exposed() {
        let safe: States = vec![(Toggle::Wine, Some(false)), (Toggle::Firewall, Some(true))];
        assert_eq!(
            waybar(&safe),
            r#"{"text":"All safe","class":"safe","tooltip":"Wine: Locked\nFirewall: Active"}"#
        );
        let open: States = vec![(Toggle::Wine, Some(true)), (Toggle::Xdotool, Some(true)), (Toggle::Firewall, Some(true))];
        assert!(waybar(&open).contains(r#""text":"2 exposed","class":"exposed""#));
    }

    #[test]
    fn missing_tool_is_not_counted() {
        let list: States = vec![(Toggle::Wine, None), (Toggle::Firewall, Some(false))];
        assert_eq!(exposed(&list), 1);
        assert!(plain(&list).contains("not found"));
    }

    #[test]
    fn set_arguments() {
        assert_eq!(parse_set(&s(&["wine", "off"])), Ok((Toggle::Wine, false)));
        assert_eq!(parse_set(&s(&["firewall", "on"])), Ok((Toggle::Firewall, true)));
        assert!(parse_set(&s(&["wine"])).is_err());
        assert!(parse_set(&s(&["rm", "on"])).is_err());
        assert!(parse_set(&s(&["wine", "maybe"])).is_err());
    }

    #[test]
    fn unknown_command_opens_the_window() {
        assert_eq!(run(&s(&["settings"])), None);
        assert_eq!(run(&[]), None);
    }
}
