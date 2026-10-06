//! Add or remove the Latch applet and desklet in Cinnamon. gsettings only.
//! Files are embedded in the binary. Written to ~/.local/share on enable.

use gtk::{gio, glib, prelude::*};
use std::fs;


#[derive(Clone, Copy)]
pub enum Kind {
    Applet,
    Desklet,
}

impl Kind {
    /// Cinnamon finds extensions by uuid alone. Applet and desklet must differ.
    fn uuid(self) -> &'static str {
        match self {
            Kind::Applet => "latch@amanullah",
            Kind::Desklet => "latch-desklet@amanullah",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Kind::Applet => "enabled-applets",
            Kind::Desklet => "enabled-desklets",
        }
    }

    fn dir(self) -> &'static str {
        match self {
            Kind::Applet => "applets",
            Kind::Desklet => "desklets",
        }
    }

    fn files(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Kind::Applet => &[
                ("metadata.json", include_str!("../../../extras/cinnamon/applet/latch@amanullah/metadata.json")),
                ("applet.js", include_str!("../../../extras/cinnamon/applet/latch@amanullah/applet.js")),
            ],
            Kind::Desklet => &[
                ("metadata.json", include_str!("../../../extras/cinnamon/desklet/latch-desklet@amanullah/metadata.json")),
                ("desklet.js", include_str!("../../../extras/cinnamon/desklet/latch-desklet@amanullah/desklet.js")),
                ("stylesheet.css", include_str!("../../../extras/cinnamon/desklet/latch-desklet@amanullah/stylesheet.css")),
            ],
        }
    }
}

fn settings() -> Option<gio::Settings> {
    let source = gio::SettingsSchemaSource::default()?;
    source.lookup("org.cinnamon", true)?;
    Some(gio::Settings::new("org.cinnamon"))
}

pub fn available() -> bool {
    settings().is_some()
}

fn entries(kind: Kind) -> Vec<String> {
    settings().map(|s| s.strv(kind.key()).iter().map(|g| g.to_string()).collect()).unwrap_or_default()
}

fn is_ours(kind: Kind, entry: &str) -> bool {
    let parts: Vec<&str> = entry.split(':').collect();
    match kind {
        Kind::Applet => parts.get(3) == Some(&kind.uuid()),
        Kind::Desklet => parts.first() == Some(&kind.uuid()),
    }
}

pub fn is_enabled(kind: Kind) -> bool {
    entries(kind).iter().any(|e| is_ours(kind, e))
}

fn ensure_files(kind: Kind) -> Result<(), String> {
    let dir = glib::user_data_dir().join("cinnamon").join(kind.dir()).join(kind.uuid());
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (name, body) in kind.files() {
        fs::write(dir.join(name), body).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Next free numeric id across all entries (last field for applets, second for desklets).
fn next_id(kind: Kind, list: &[String]) -> u32 {
    let pick = |e: &String| {
        let parts: Vec<&str> = e.split(':').collect();
        let at = if matches!(kind, Kind::Applet) { 4 } else { 1 };
        parts.get(at).and_then(|n| n.parse::<u32>().ok())
    };
    list.iter().filter_map(pick).max().map_or(0, |m| m + 1)
}

/// Applet goes at the end of the right zone of the last panel that has one.
fn applet_entry(list: &[String], id: u32) -> String {
    let uuid = Kind::Applet.uuid();
    let right: Vec<Vec<&str>> =
        list.iter().map(|e| e.split(':').collect::<Vec<_>>()).filter(|p| p.get(1) == Some(&"right")).collect();
    let panel = right.last().map_or("panel1", |p| p[0]);
    let order = right
        .iter()
        .filter(|p| p[0] == panel)
        .filter_map(|p| p.get(2).and_then(|n| n.parse::<u32>().ok()))
        .max()
        .map_or(0, |m| m + 1);
    format!("{panel}:right:{order}:{uuid}:{id}")
}

pub fn enable(kind: Kind) -> Result<(), String> {
    let s = settings().ok_or("Cinnamon not found")?;
    let mut list = entries(kind);
    if list.iter().any(|e| is_ours(kind, e)) {
        return Ok(());
    }
    ensure_files(kind)?;
    let id = next_id(kind, &list);
    list.push(match kind {
        Kind::Applet => applet_entry(&list, id),
        Kind::Desklet => format!("{}:{id}:340:60", kind.uuid()),
    });
    let refs: Vec<&str> = list.iter().map(String::as_str).collect();
    s.set_strv(kind.key(), refs.as_slice()).map_err(|e| e.to_string())?;
    gio::Settings::sync();
    Ok(())
}

pub fn disable(kind: Kind) -> Result<(), String> {
    let s = settings().ok_or("Cinnamon not found")?;
    let list: Vec<String> = entries(kind).into_iter().filter(|e| !is_ours(kind, e)).collect();
    let refs: Vec<&str> = list.iter().map(String::as_str).collect();
    s.set_strv(kind.key(), refs.as_slice()).map_err(|e| e.to_string())?;
    gio::Settings::sync();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applet_goes_last_in_right_zone() {
        let list = vec![
            "panel1:left:0:menu@x:0".to_string(),
            "panel2:right:3:a@x:1".to_string(),
            "panel2:right:7:b@x:2".to_string(),
        ];
        assert_eq!(applet_entry(&list, 3), "panel2:right:8:latch@amanullah:3");
        assert_eq!(next_id(Kind::Applet, &list), 3);
        assert_eq!(applet_entry(&[], 0), "panel1:right:0:latch@amanullah:0");
    }

    #[test]
    fn ours() {
        assert!(is_ours(Kind::Applet, "panel1:right:0:latch@amanullah:3"));
        assert!(is_ours(Kind::Desklet, "latch-desklet@amanullah:3:10:10"));
        assert!(!is_ours(Kind::Desklet, "clock@cinnamon.org:3:10:10"));
    }
}
