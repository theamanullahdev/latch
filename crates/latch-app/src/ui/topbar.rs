//! Own title bar (client side), so it follows our theme. Title left, search right.

use gtk::prelude::*;

pub fn build() -> (gtk::HeaderBar, gtk::SearchEntry) {
    let bar = gtk::HeaderBar::new();
    bar.set_show_close_button(true);
    bar.set_title(Some("Latch"));

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search"));
    search.set_width_chars(26);
    bar.pack_end(&search);
    (bar, search)
}
