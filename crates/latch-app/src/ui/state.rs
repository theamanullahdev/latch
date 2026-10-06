//! On/off per toggle. Drives status bar text.

use gtk::prelude::*;
use latch_core::{probe, Toggle};
use std::{cell::Cell, rc::Rc};

pub struct State {
    on: Cell<[bool; 3]>,
    label: gtk::Label,
}

impl State {
    /// Start from real state. Unknown = off.
    pub fn new(label: gtk::Label) -> Rc<Self> {
        let on = Toggle::ALL.map(|t| probe::is_on(t).unwrap_or(false));
        let state = Rc::new(Self { on: Cell::new(on), label });
        state.refresh();
        state
    }

    pub fn is_on(&self, t: Toggle) -> bool {
        self.on.get()[t.index()]
    }

    pub fn set(&self, t: Toggle, on: bool) {
        let mut all = self.on.get();
        all[t.index()] = on;
        self.on.set(all);
        self.refresh();
    }

    fn refresh(&self) {
        let risky = Toggle::ALL.iter().filter(|t| t.is_risky(self.is_on(**t))).count();
        let text = match risky {
            0 => "All safe".to_string(),
            n => format!("{n} exposed"),
        };
        self.label.set_text(&text);
        let ctx = self.label.parent().map(|p| p.style_context());
        if let Some(ctx) = ctx {
            if risky > 0 { ctx.add_class("risky") } else { ctx.remove_class("risky") }
        }
    }
}
