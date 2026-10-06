//! Settings: theme, app name and icon, install / remove of each piece.

use crate::{branding, cinnamon::{self, Kind}, config, menu, setup, theme::{self, Mode}};
use gtk::{glib, prelude::*};
use std::rc::Rc;

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
    if config::load().theme == Mode::System {
        system.set_active(true);
    }
    for (button, mode) in [(&latch, Mode::Latch), (&system, Mode::System)] {
        button.connect_toggled(move |b| {
            if b.is_active() {
                theme::apply(mode);
                config::update(|c| c.theme = mode);
            }
        });
        col.pack_start(button, false, false, 0);
    }
    col
}

fn row(item: Item) -> (gtk::Box, Rc<dyn Fn()>) {
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

    let item = Rc::new(item);
    let refresh: Rc<dyn Fn()> = {
        let (button, item) = (button.clone(), item.clone());
        Rc::new(move || {
            button.set_label(if (item.installed)() { "Remove" } else { "Install" });
            button.set_sensitive((item.usable)());
        })
    };
    refresh();
    let again = refresh.clone();
    button.connect_clicked(move |b| {
        let (b, err, item, refresh) = (b.clone(), err.clone(), item.clone(), again.clone());
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
    (line, refresh)
}

fn caption(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_halign(gtk::Align::Start);
    l.set_width_chars(10);
    l.set_xalign(0.0);
    l
}

/// App name (saved on Enter or leaving the box) and icon (pick a file, or reset).
fn appearance() -> gtk::Box {
    let col = gtk::Box::new(gtk::Orientation::Vertical, 8);

    let name_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let entry = gtk::Entry::new();
    entry.set_text(&config::load().name);
    entry.set_max_length(24);
    entry.set_width_chars(22);
    entry.connect_activate(|e| branding::set_name(&e.text()));
    entry.connect_focus_out_event(|e, _| {
        branding::set_name(&e.text());
        glib::Propagation::Proceed
    });
    name_row.pack_start(&caption("App name"), false, false, 0);
    name_row.pack_start(&entry, false, false, 0);

    let icon_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let choose = gtk::Button::with_label("Choose...");
    let reset = gtk::Button::with_label("Reset");
    for b in [&choose, &reset] {
        b.style_context().add_class("test-btn");
    }
    let err = gtk::Label::new(None);
    err.style_context().add_class("error");
    err.set_halign(gtk::Align::Start);
    err.set_line_wrap(true);
    err.set_xalign(0.0);
    choose.connect_clicked({
        let err = err.clone();
        move |b| {
            let parent = b.toplevel().and_then(|w| w.downcast::<gtk::Window>().ok());
            let dialog = gtk::FileChooserNative::new(
                Some("Choose an icon"),
                parent.as_ref(),
                gtk::FileChooserAction::Open,
                Some("Choose"),
                Some("Cancel"),
            );
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Images"));
            filter.add_pixbuf_formats();
            filter.add_mime_type("image/svg+xml");
            dialog.add_filter(filter);
            if dialog.run() == gtk::ResponseType::Accept {
                if let Some(path) = dialog.filename() {
                    err.set_text(&branding::set_icon(&path).err().unwrap_or_default());
                }
            }
        }
    });
    reset.connect_clicked({
        let err = err.clone();
        move |_| {
            branding::reset_icon();
            err.set_text("");
        }
    });
    icon_row.pack_start(&caption("App icon"), false, false, 0);
    icon_row.pack_start(&choose, false, false, 0);
    icon_row.pack_start(&reset, false, false, 0);

    col.pack_start(&name_row, false, false, 0);
    col.pack_start(&icon_row, false, false, 0);
    col.pack_start(&err, false, false, 0);
    col
}

pub fn build() -> gtk::ScrolledWindow {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    page.set_border_width(24);

    let title = gtk::Label::new(Some("Settings"));
    title.set_halign(gtk::Align::Start);
    title.style_context().add_class("panel-title");

    page.pack_start(&title, false, false, 0);
    page.pack_start(&heading("THEME"), false, false, 6);
    page.pack_start(&theme_choice(), false, false, 0);
    page.pack_start(&heading("APPEARANCE"), false, false, 6);
    page.pack_start(&appearance(), false, false, 0);
    page.pack_start(&heading("INSTALL"), false, false, 6);
    let mut refreshers = Vec::new();
    for item in items() {
        let (line, refresh) = row(item);
        page.pack_start(&line, false, false, 4);
        refreshers.push(refresh);
    }
    // State can change elsewhere (first switch flip installs the helper).
    page.connect_map(move |_| refreshers.iter().for_each(|r| r()));

    let scroll = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.add(&page);
    scroll
}
