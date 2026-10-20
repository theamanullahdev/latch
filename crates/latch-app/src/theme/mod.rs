//! Latch mode: dark base theme + latch.css. System mode: user theme untouched.
//! base.css (spacing, sizes, state colors) applies in both.

use gtk::{gdk, prelude::*};
use std::cell::RefCell;

const CSS: &str = include_str!("latch.css");
const BASE: &str = include_str!("base.css");
const BASE_THEME: &str = "Adwaita-dark";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Latch,
    System,
}

struct Slot {
    provider: gtk::CssProvider,
    system_theme: Option<String>,
    system_dark: bool,
    applied: Option<Mode>,
}

thread_local! {
    static SLOT: RefCell<Option<Slot>> = const { RefCell::new(None) };
}

pub fn init(mode: Mode) {
    if let Some(screen) = gdk::Screen::default() {
        let base = gtk::CssProvider::new();
        base.load_from_data(BASE.as_bytes()).expect("base.css");
        gtk::StyleContext::add_provider_for_screen(&screen, &base, gtk::STYLE_PROVIDER_PRIORITY_USER);
    }
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS.as_bytes()).expect("latch.css");
    let system_theme = gtk::Settings::default()
        .and_then(|s| s.gtk_theme_name())
        .map(|n| n.to_string());
    let system_dark = gtk::Settings::default().is_some_and(|s| s.is_gtk_application_prefer_dark_theme());
    SLOT.with(|s| *s.borrow_mut() = Some(Slot { provider, system_theme, system_dark, applied: None }));
    apply(mode);
}

pub fn apply(mode: Mode) {
    let (Some(screen), Some(settings)) = (gdk::Screen::default(), gtk::Settings::default()) else {
        return;
    };
    SLOT.with(|s| {
        let mut s = s.borrow_mut();
        let Some(slot) = s.as_mut() else { return };
        if slot.applied == Some(mode) {
            return;
        }
        if slot.applied == Some(Mode::Latch) {
            gtk::StyleContext::remove_provider_for_screen(&screen, &slot.provider);
        }
        match mode {
            Mode::Latch => {
                settings.set_gtk_theme_name(Some(BASE_THEME));
                // Systems without the Adwaita-dark theme still get GTK's built-in dark variant.
                settings.set_gtk_application_prefer_dark_theme(true);
                gtk::StyleContext::add_provider_for_screen(
                    &screen,
                    &slot.provider,
                    gtk::STYLE_PROVIDER_PRIORITY_USER,
                );
            }
            Mode::System => {
                settings.set_gtk_theme_name(slot.system_theme.as_deref());
                settings.set_gtk_application_prefer_dark_theme(slot.system_dark);
            }
        }
        slot.applied = Some(mode);
    });
}
