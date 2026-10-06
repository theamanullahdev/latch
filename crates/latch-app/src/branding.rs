//! Custom app name and icon. Saved in config, applied to windows and the menu entry.

use crate::{config, menu, ui::topbar};
use gtk::{gdk_pixbuf::Pixbuf, glib, prelude::*};
use std::{fs, io::Cursor, path::Path};

const ICON: &str = include_str!("../../../extras/packaging/icons/latch.svg");
const MAX_NAME: usize = 24;

/// Title, headerbar title and icon of every app window.
pub fn apply() {
    let cfg = config::load();
    let pixbuf = match &cfg.icon {
        Some(path) => Pixbuf::from_file(path).ok(),
        None => Pixbuf::from_read(Cursor::new(ICON.as_bytes())).ok(),
    };
    for widget in gtk::Window::list_toplevels() {
        let Ok(win) = widget.downcast::<gtk::ApplicationWindow>() else { continue };
        win.set_title(&cfg.name);
        if let Some(bar) = win.titlebar().and_then(|t| t.downcast::<gtk::HeaderBar>().ok()) {
            let title = bar.children().into_iter().find(|w| w.widget_name() == topbar::TITLE_ID);
            if let Some(label) = title.and_then(|w| w.downcast::<gtk::Label>().ok()) {
                label.set_text(&cfg.name);
            }
        }
        win.set_icon(pixbuf.as_ref());
    }
}

pub fn set_name(raw: &str) {
    let clean: String = raw.lines().next().unwrap_or("").trim().chars().take(MAX_NAME).collect();
    let name = if clean.is_empty() { config::DEFAULT_NAME.to_string() } else { clean };
    config::update(|c| c.name = name);
    changed();
}

/// Copy into the app data dir, so the file survives if the original moves.
pub fn set_icon(src: &Path) -> Result<(), String> {
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let dir = glib::user_data_dir().join("latch");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(format!("icon.{ext}"));
    fs::copy(src, &dest).map_err(|e| e.to_string())?;
    if Pixbuf::from_file(&dest).is_err() {
        let _ = fs::remove_file(&dest);
        return Err("That file is not a usable image.".into());
    }
    config::update(|c| c.icon = Some(dest));
    changed();
    Ok(())
}

pub fn reset_icon() {
    if let Some(icon) = config::load().icon {
        let _ = fs::remove_file(icon);
    }
    config::update(|c| c.icon = None);
    changed();
}

fn changed() {
    apply();
    if menu::is_installed() {
        let _ = menu::install();
    }
}
