//! Timed unlock. Ticks each second, flips the switch off at zero.
//! Deadline is saved, so a restart or a close cannot leave it open.

use crate::{backend, config};
use gtk::{glib, prelude::*};
use latch_core::Toggle;
use std::{cell::RefCell, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

pub const CHOICES: [(&str, u32); 5] =
    [("Off", 0), ("1 minute", 60), ("5 minutes", 300), ("15 minutes", 900), ("1 hour", 3600)];

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

pub struct AutoLock {
    t: Toggle,
    switch: gtk::Switch,
    /// Seconds left each tick. None = no timer.
    show: Rc<dyn Fn(Option<u64>)>,
    source: RefCell<Option<glib::SourceId>>,
}

impl AutoLock {
    pub fn new(t: Toggle, switch: gtk::Switch, show: Rc<dyn Fn(Option<u64>)>) -> Rc<Self> {
        Rc::new(Self { t, switch, show, source: RefCell::new(None) })
    }

    /// Switch just turned on: start the saved delay, if any.
    pub fn arm(self: &Rc<Self>) {
        let secs = config::load().autolock[self.t.index()];
        if secs == 0 {
            return self.disarm();
        }
        let due = now() + u64::from(secs);
        config::update(|c| c.deadline[self.t.index()] = due);
        self.run(due);
    }

    /// App start: pick up a saved deadline. Overdue = lock now.
    pub fn resume(self: &Rc<Self>, is_on: bool) {
        let due = config::load().deadline[self.t.index()];
        if due == 0 {
            return;
        }
        if !is_on {
            return self.disarm();
        }
        if due <= now() {
            let switch = self.switch.clone();
            glib::idle_add_local_once(move || switch.set_active(false));
        } else {
            self.run(due);
        }
    }

    pub fn disarm(&self) {
        self.stop_source();
        config::update(|c| c.deadline[self.t.index()] = 0);
        (self.show)(None);
    }

    fn stop_source(&self) {
        if let Some(id) = self.source.borrow_mut().take() {
            id.remove();
        }
    }

    fn run(self: &Rc<Self>, due: u64) {
        self.stop_source();
        (self.show)(Some(due.saturating_sub(now())));
        let me = self.clone();
        let id = glib::timeout_add_seconds_local(1, move || {
            let left = due.saturating_sub(now());
            if left == 0 {
                me.source.borrow_mut().take();
                me.switch.set_active(false);
                return glib::ControlFlow::Break;
            }
            (me.show)(Some(left));
            glib::ControlFlow::Continue
        });
        *self.source.borrow_mut() = Some(id);
    }
}

/// Window closing: lock whatever is still on a timer. Safe direction, no prompt.
pub fn lock_pending() {
    let cfg = config::load();
    for t in Toggle::ALL.into_iter().filter(|t| t.timed() && cfg.deadline[t.index()] != 0) {
        if backend::apply_blocking(t, false).is_ok() {
            config::update(|c| c.deadline[t.index()] = 0);
        }
    }
}
