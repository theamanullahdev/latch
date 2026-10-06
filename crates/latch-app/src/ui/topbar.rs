//! Own title bar (client side), so it follows our theme. Title left, search right.
//! Title is a left-packed label (GTK's centered title gets cut by the search box).

use gtk::prelude::*;

pub const TITLE_ID: &str = "latch-title";

pub fn build(name: &str) -> (gtk::HeaderBar, gtk::SearchEntry) {
    let bar = gtk::HeaderBar::new();
    bar.set_show_close_button(true);
    // Empty center widget: stops GTK drawing a second, centered title.
    bar.set_custom_title(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    let title = gtk::Label::new(Some(name));
    title.set_widget_name(TITLE_ID);
    title.style_context().add_class("latch-title");
    bar.pack_start(&title);

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search"));
    search.set_width_chars(26);
    bar.pack_end(&search);
    (bar, search)
}
