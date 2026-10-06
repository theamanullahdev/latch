//! One page per toggle: orb, name, state, switch, test.

use super::{orb::Orb, state::State};
use crate::{backend, selftest};
use gtk::{glib, prelude::*};
use latch_core::{probe, Toggle};
use std::{cell::Cell, rc::Rc};

fn label(text: &str, class: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.style_context().add_class(class);
    l.set_justify(gtk::Justification::Center);
    l.set_line_wrap(true);
    l
}

fn set_class(w: &impl IsA<gtk::Widget>, class: &str, on: bool) {
    let ctx = w.style_context();
    if on { ctx.add_class(class) } else { ctx.remove_class(class) }
}

pub fn build(t: Toggle, state: Rc<State>) -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 8);
    page.set_valign(gtk::Align::Center);
    page.set_border_width(20);

    let on = state.is_on(t);
    let installed = probe::is_on(t).is_some();
    let orb = Orb::new(t, t.is_risky(on));
    let status = label("", "state");
    let msg = label("", "error");
    let switch = gtk::Switch::new();
    switch.set_halign(gtk::Align::Center);
    switch.set_active(on);
    switch.set_sensitive(installed);
    set_class(&switch, "safe", t == Toggle::Firewall);

    let show = {
        let (status, orb) = (status.clone(), orb.clone());
        move |on: bool| {
            let risky = t.is_risky(on);
            status.set_text(t.state_text(on));
            set_class(&status, "risky", risky);
            orb.set_risky(risky);
        }
    };
    show(on);
    if !installed {
        status.set_text("Not found");
    }

    page.pack_start(&orb.area, false, false, 4);
    page.pack_start(&label(t.name(), "panel-title"), false, false, 0);
    page.pack_start(&status, false, false, 0);
    page.pack_start(&label(t.blurb(), "dim"), false, false, 0);
    page.pack_start(&switch, false, false, 10);
    page.pack_start(&msg, false, false, 0);
    page.pack_start(&test_row(t, state.clone()), false, false, 4);

    let syncing = Rc::new(Cell::new(false));
    switch.connect_active_notify(move |s| {
        if syncing.get() {
            return;
        }
        let want = s.is_active();
        s.set_sensitive(false);
        orb.set_busy(true);
        msg.set_text("");
        let (s, orb, msg, show, state, syncing) =
            (s.clone(), orb.clone(), msg.clone(), show.clone(), state.clone(), syncing.clone());
        glib::spawn_future_local(async move {
            let result = backend::apply(t, want).await;
            let now = probe::is_on(t).unwrap_or(!want);
            syncing.set(true);
            s.set_active(now);
            syncing.set(false);
            s.set_sensitive(true);
            orb.set_busy(false);
            state.set(t, now);
            show(now);
            if let Err(e) = result {
                msg.set_text(&e);
            }
        });
    });
    page
}

/// Button + output right under the switch. Verdict = tool state vs switch.
fn test_row(t: Toggle, state: Rc<State>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let button = gtk::Button::with_label("Run test");
    button.style_context().add_class("test-btn");
    button.set_halign(gtk::Align::Center);
    let lines = label("", "result");
    let verdict = label("", "result");

    row.pack_start(&button, false, false, 0);
    row.pack_start(&lines, false, false, 0);
    row.pack_start(&verdict, false, false, 0);

    button.connect_clicked(move |b| {
        let (b, lines, verdict, state) = (b.clone(), lines.clone(), verdict.clone(), state.clone());
        b.set_sensitive(false);
        b.set_label("Testing...");
        lines.set_text(if t == Toggle::Wine { "first run builds a private prefix, ~20 s" } else { "" });
        verdict.set_text("");
        glib::spawn_future_local(async move {
            let out = selftest::run(t).await;
            let matches = out.works == state.is_on(t);
            lines.set_text(&out.lines.join("\n"));
            verdict.set_text(if matches { "matches the switch" } else { "does not match the switch" });
            set_class(&verdict, "ok", matches);
            set_class(&verdict, "bad", !matches);
            b.set_label("Run test");
            b.set_sensitive(true);
        });
    });
    row
}
