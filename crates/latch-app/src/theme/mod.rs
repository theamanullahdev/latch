//! Latch mode: dark base theme + latch.css. System mode: user theme untouched.

use gtk::{gdk, prelude::*};
use std::cell::RefCell;

const CSS: &str = include_str!("latch.css");
const BASE_THEME: &str = "Adwaita-dark";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Latch,
    System,
}

struct Slot {
    provider: gtk::CssProvider,
    system_theme: Option<String>,
    applied: Option<Mode>,
}

thread_local! {
    static SLOT: RefCell<Option<Slot>> = const { RefCell::new(None) };
}

pub fn init(mode: Mode) {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS.as_bytes()).expect("latch.css");
    let system_theme = gtk::Settings::default()
        .and_then(|s| s.gtk_theme_name())
        .map(|n| n.to_string());
    SLOT.with(|s| *s.borrow_mut() = Some(Slot { provider, system_theme, applied: None }));
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
                gtk::StyleContext::add_provider_for_screen(
                    &screen,
                    &slot.provider,
                    gtk::STYLE_PROVIDER_PRIORITY_USER,
                );
            }
            Mode::System => settings.set_gtk_theme_name(slot.system_theme.as_deref()),
        }
        slot.applied = Some(mode);
    });
}
