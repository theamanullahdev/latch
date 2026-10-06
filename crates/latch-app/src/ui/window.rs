//! Main window: topbar, sidebar + panel, statusbar.

use super::{panel, settings, sidebar, state::State, statusbar, topbar};
use gtk::prelude::*;
use latch_core::Toggle;

pub fn build(app: &gtk::Application, page: Option<&str>) -> gtk::ApplicationWindow {
    let win = gtk::ApplicationWindow::new(app);
    win.set_title("Latch");
    win.set_default_size(900, 560);
    win.set_titlebar(Some(&topbar::build()));

    let summary = gtk::Label::new(None);
    let bar = statusbar::build(&summary);
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

    let middle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    middle.pack_start(&sidebar::build(&stack), false, false, 0);
    middle.pack_start(&stack, true, true, 0);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.pack_start(&middle, true, true, 0);
    root.pack_start(&bar, false, false, 0);
    win.add(&root);
    win
}
