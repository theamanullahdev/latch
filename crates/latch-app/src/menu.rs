//! Menu entry for this user. Desktop file + icon embedded, written to ~/.local/share.

use crate::config;
use gtk::glib;
use std::{env, fs, path::PathBuf, process::Command};

const DESKTOP: &str = include_str!("../../../extras/packaging/desktop/latch.desktop");
const ICON: &str = include_str!("../../../extras/packaging/icons/latch.svg");

fn desktop_path() -> PathBuf {
    glib::user_data_dir().join("applications").join("latch.desktop")
}

fn icon_dir() -> PathBuf {
    glib::user_data_dir().join("icons").join("hicolor")
}

pub fn is_installed() -> bool {
    desktop_path().exists()
}

pub fn install() -> Result<(), String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let apps = desktop_path();
    let icons = icon_dir().join("scalable").join("apps");
    fs::create_dir_all(apps.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::create_dir_all(&icons).map_err(|e| e.to_string())?;
    let cfg = config::load();
    let icon = cfg.icon.map_or("latch".to_string(), |p| p.display().to_string());
    let entry = DESKTOP
        .replace("Exec=latch", &format!("Exec=\"{}\"", exe.display()))
        .replacen("Name=Latch", &format!("Name={}", cfg.name), 1)
        .replace("Icon=latch", &format!("Icon={icon}"));
    fs::write(&apps, entry).map_err(|e| e.to_string())?;
    fs::write(icons.join("latch.svg"), ICON).map_err(|e| e.to_string())?;
    refresh_icons();
    Ok(())
}

pub fn remove() -> Result<(), String> {
    let _ = fs::remove_file(desktop_path());
    let _ = fs::remove_file(icon_dir().join("scalable").join("apps").join("latch.svg"));
    refresh_icons();
    Ok(())
}

fn refresh_icons() {
    let _ = Command::new("gtk-update-icon-cache").arg("-f").arg("-t").arg(icon_dir()).output();
}
