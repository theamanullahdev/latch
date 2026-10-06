//! Bottom bar. Holds the summary label.

use gtk::prelude::*;

pub fn build(summary: &gtk::Label) -> gtk::Box {
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    bar.style_context().add_class("statusbar");
    bar.pack_start(summary, false, false, 0);
    bar
}
