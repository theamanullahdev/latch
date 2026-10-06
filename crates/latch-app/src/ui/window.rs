//! Main window: topbar, sidebar + panel, statusbar.

use super::{autolock, panel, settings, sidebar, state::State, statusbar, topbar};
use gtk::prelude::*;
use latch_core::Toggle;

pub fn build(app: &gtk::Application, page: Option<&str>) -> gtk::ApplicationWindow {
    let win = gtk::ApplicationWindow::new(app);
    let name = crate::config::load().name;
    win.set_title(&name);
    win.set_default_size(900, 640);
    let (bar, search) = topbar::build(&name);
    win.set_titlebar(Some(&bar));

    let summary = gtk::Label::new(None);
    let status = statusbar::build(&summary);
    let state = State::new(summary);

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_transition_duration(240);
    for t in Toggle::ALL {
        stack.add_titled(&panel::build(t, state.clone()), t.id(), t.name());
    }
    stack.add_titled(&settings::build(), "settings", "Settings");
    if let Some(name) = page {
        stack.show_all();
        stack.set_visible_child_name(name);
    }

    search.connect_search_changed({
        let stack = stack.clone();
        move |s| {
            if let Some(page) = find_page(&s.text().to_lowercase()) {
                stack.set_visible_child_name(page);
            }
        }
    });

    let middle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    middle.pack_start(&sidebar::build(&stack), false, false, 0);
    middle.pack_start(&stack, true, true, 0);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.pack_start(&middle, true, true, 0);
    root.pack_start(&status, false, false, 0);
    win.add(&root);
    win.connect_delete_event(|_, _| {
        autolock::lock_pending();
        gtk::glib::Propagation::Proceed
    });
    win
}

const SETTINGS_WORDS: [&str; 6] = ["settings", "theme", "install", "applet", "desklet", "helper"];

/// First page whose name, blurb or keywords contain the query.
fn find_page(q: &str) -> Option<&'static str> {
    if q.is_empty() {
        return None;
    }
    let toggle = Toggle::ALL
        .iter()
        .find(|t| t.name().to_lowercase().contains(q) || t.blurb().to_lowercase().contains(q));
    match toggle {
        Some(t) => Some(t.id()),
        None => SETTINGS_WORDS.iter().any(|w| w.contains(q)).then_some("settings"),
    }
}

#[cfg(test)]
mod tests {
    use super::find_page;

    #[test]
    fn search_finds_pages() {
        assert_eq!(find_page("fire"), Some("firewall"));
        assert_eq!(find_page("xdo"), Some("xdotool"));
        assert_eq!(find_page("exe"), Some("wine"));
        assert_eq!(find_page("them"), Some("settings"));
        assert_eq!(find_page(""), None);
        assert_eq!(find_page("zzz"), None);
    }
}
