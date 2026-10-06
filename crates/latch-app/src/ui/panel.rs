//! One page per toggle: orb, name, state, switch, auto-lock, test.

use super::{autolock::{self, AutoLock}, orb::Orb, state::State};
use crate::{backend, config, selftest};
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

    // Countdown text while a timer runs. No timer = plain state text.
    let countdown: Rc<dyn Fn(Option<u64>)> = {
        let (status, switch, show) = (status.clone(), switch.clone(), show.clone());
        Rc::new(move |left| match left {
            Some(s) => status.set_text(&format!("{} - locks in {}:{:02}", t.state_text(true), s / 60, s % 60)),
            None => show(switch.is_active()),
        })
    };
    let timer = AutoLock::new(t, switch.clone(), countdown);

    page.pack_start(&orb.area, false, false, 4);
    page.pack_start(&label(t.name(), "panel-title"), false, false, 0);
    page.pack_start(&status, false, false, 0);
    page.pack_start(&label(t.blurb(), "dim"), false, false, 0);
    page.pack_start(&switch, false, false, 8);
    if t.timed() {
        page.pack_start(&autolock_row(t), false, false, 0);
    }
    page.pack_start(&msg, false, false, 0);
    page.pack_start(&test_row(t, state.clone()), false, false, 4);

    let syncing = Rc::new(Cell::new(false));
    let handler_timer = timer.clone();
    switch.connect_active_notify(move |s| {
        if syncing.get() {
            return;
        }
        let want = s.is_active();
        s.set_sensitive(false);
        orb.set_busy(true);
        msg.set_text("");
        let (s, orb, msg, show, state, syncing, timer) =
            (s.clone(), orb.clone(), msg.clone(), show.clone(), state.clone(), syncing.clone(), handler_timer.clone());
        glib::spawn_future_local(async move {
            let result = backend::apply(t, want).await;
            let now = probe::is_on(t).unwrap_or(!want);
            syncing.set(true);
            s.set_active(now);
            syncing.set(false);
            s.set_sensitive(true);
            s.grab_focus();
            orb.set_busy(false);
            state.set(t, now);
            show(now);
            if want && now && t.timed() {
                timer.arm();
            } else if !now {
                timer.disarm();
            }
            if let Err(e) = result {
                msg.set_text(&e);
            }
        });
    });
    timer.resume(on);
    page
}

/// "Auto-lock: [Off | 1 minute | ...]". Saved. Applies at the next unlock.
fn autolock_row(t: Toggle) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.set_halign(gtk::Align::Center);
    let combo = gtk::ComboBoxText::new();
    for (name, _) in autolock::CHOICES {
        combo.append_text(name);
    }
    let saved = config::load().autolock[t.index()];
    combo.set_active(Some(autolock::CHOICES.iter().position(|c| c.1 == saved).unwrap_or(0) as u32));
    combo.set_tooltip_text(Some("Locks again by itself. Closing Latch locks it early."));
    combo.connect_changed(move |c| {
        let secs = c.active().and_then(|i| autolock::CHOICES.get(i as usize)).map_or(0, |c| c.1);
        config::update(|cfg| cfg.autolock[t.index()] = secs);
    });
    row.pack_start(&label("Auto-lock", "dim"), false, false, 0);
    row.pack_start(&combo, false, false, 0);
    row
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
