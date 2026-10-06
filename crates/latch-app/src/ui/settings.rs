//! Settings: theme choice, plus install / remove of each piece.

use crate::{cinnamon::{self, Kind}, config, menu, setup, theme::{self, Mode}};
use gtk::{glib, prelude::*};

type Op = fn() -> Result<(), String>;

struct Item {
    title: &'static str,
    note: &'static str,
    installed: fn() -> bool,
    install: Op,
    remove: Op,
    usable: fn() -> bool,
}

fn always() -> bool {
    true
}

fn items() -> [Item; 4] {
    [
        Item { title: "System helper", note: "Lets Latch switch things. One admin prompt.",
               installed: setup::is_installed, install: setup::install, remove: setup::remove, usable: always },
        Item { title: "Menu entry", note: "Start Latch from the app menu.",
               installed: menu::is_installed, install: menu::install, remove: menu::remove, usable: always },
        Item { title: "Panel applet", note: "Switches in the Cinnamon panel.",
               installed: || cinnamon::is_enabled(Kind::Applet),
               install: || cinnamon::enable(Kind::Applet), remove: || cinnamon::disable(Kind::Applet),
               usable: cinnamon::available },
        Item { title: "Desktop desklet", note: "A Latch card on the desktop.",
               installed: || cinnamon::is_enabled(Kind::Desklet),
               install: || cinnamon::enable(Kind::Desklet), remove: || cinnamon::disable(Kind::Desklet),
               usable: cinnamon::available },
    ]
}

fn heading(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_halign(gtk::Align::Start);
    l.style_context().add_class("section");
    l
}

fn radio(group: Option<&gtk::RadioButton>, text: &str) -> gtk::RadioButton {
    match group {
        Some(g) => gtk::RadioButton::with_label_from_widget(g, text),
        None => gtk::RadioButton::with_label(text),
    }
}

fn theme_choice() -> gtk::Box {
    let col = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let latch = radio(None, "Latch (default)");
    let system = radio(Some(&latch), "System (Cinnamon theme)");
    if config::load_theme() == Mode::System {
        system.set_active(true);
    }
    for (button, mode) in [(&latch, Mode::Latch), (&system, Mode::System)] {
        button.connect_toggled(move |b| {
            if b.is_active() {
                theme::apply(mode);
                config::save_theme(mode);
            }
        });
        col.pack_start(button, false, false, 0);
    }
    col
}

fn row(item: Item) -> gtk::Box {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let title = gtk::Label::new(Some(item.title));
    let note = gtk::Label::new(Some(item.note));
    let err = gtk::Label::new(None);
    for (l, class) in [(&title, ""), (&note, "dim"), (&err, "error")] {
        l.set_halign(gtk::Align::Start);
        if !class.is_empty() {
            l.style_context().add_class(class);
        }
        text.pack_start(l, false, false, 0);
    }
    let button = gtk::Button::new();
    button.style_context().add_class("test-btn");
    button.set_valign(gtk::Align::Center);
    line.pack_start(&text, true, true, 0);
    line.pack_end(&button, false, false, 0);

    let item = std::rc::Rc::new(item);
    let refresh = {
        let (button, item) = (button.clone(), item.clone());
        move || {
            button.set_label(if (item.installed)() { "Remove" } else { "Install" });
            button.set_sensitive((item.usable)());
        }
    };
    refresh();
    button.connect_clicked(move |b| {
        let (b, err, item, refresh) = (b.clone(), err.clone(), item.clone(), refresh.clone());
        let op: Op = if (item.installed)() { item.remove } else { item.install };
        b.set_sensitive(false);
        err.set_text("");
        glib::spawn_future_local(async move {
            if let Ok(Err(e)) = gtk::gio::spawn_blocking(op).await {
                err.set_text(&e);
            }
            refresh();
        });
    });
    line
}

pub fn build() -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    page.set_border_width(24);

    let title = gtk::Label::new(Some("Settings"));
    title.set_halign(gtk::Align::Start);
    title.style_context().add_class("panel-title");

    page.pack_start(&title, false, false, 0);
    page.pack_start(&heading("THEME"), false, false, 6);
    page.pack_start(&theme_choice(), false, false, 0);
    page.pack_start(&heading("INSTALL"), false, false, 6);
    for item in items() {
        page.pack_start(&row(item), false, false, 4);
    }
    page
}
