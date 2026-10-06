//! Animated badge. Ring closed + green = safe. Ring opens, turns amber, spins = exposed.
//! Center glyph per toggle: wine glass (liquid), cursor, shield.

use gtk::{cairo, glib, prelude::*};
use latch_core::Toggle;
use std::{cell::Cell, f64::consts::TAU, rc::Rc};

type Rgb = (f64, f64, f64);
const SAFE: Rgb = (0.435, 0.682, 0.310);
const WARN: Rgb = (0.878, 0.627, 0.188);
const DISC: Rgb = (0.145, 0.137, 0.125);

#[derive(Clone)]
pub struct Orb {
    pub area: gtk::DrawingArea,
    target: Rc<Cell<f64>>,
    busy: Rc<Cell<bool>>,
}

impl Orb {
    pub fn new(kind: Toggle, risky: bool) -> Self {
        let area = gtk::DrawingArea::new();
        area.set_size_request(160, 160);
        area.set_halign(gtk::Align::Center);

        let target = Rc::new(Cell::new(risky as u8 as f64));
        let busy = Rc::new(Cell::new(false));
        let cur = Rc::new(Cell::new(target.get()));
        let phase = Rc::new(Cell::new(0.0));
        let last = Rc::new(Cell::new(0i64));

        let (c, p) = (cur.clone(), phase.clone());
        area.connect_draw(move |a, cr| {
            let size = (a.allocated_width() as f64, a.allocated_height() as f64);
            let _ = paint(cr, size, kind, c.get(), p.get());
            glib::Propagation::Proceed
        });

        let (t, b) = (target.clone(), busy.clone());
        area.add_tick_callback(move |a, clock| {
            let now = clock.frame_time();
            let dt = ((now - last.replace(now)) as f64 / 1e6).clamp(0.0, 0.1);
            let (from, to) = (cur.get(), t.get());
            let settled = (to - from).abs() < 0.002;
            cur.set(if settled { to } else { from + (to - from) * (1.0 - (-dt * 7.0).exp()) });
            let alive = to > 0.5 || b.get();
            if alive {
                phase.set(phase.get() + dt * if b.get() { 7.0 } else { 2.0 });
            }
            if alive || !settled || from != to {
                a.queue_draw();
            }
            glib::ControlFlow::Continue
        });
        Self { area, target, busy }
    }

    pub fn set_risky(&self, risky: bool) {
        self.target.set(risky as u8 as f64);
    }

    pub fn set_busy(&self, busy: bool) {
        self.busy.set(busy);
    }
}

fn mix(x: f64) -> Rgb {
    let l = |a: f64, b: f64| a + (b - a) * x;
    (l(SAFE.0, WARN.0), l(SAFE.1, WARN.1), l(SAFE.2, WARN.2))
}

fn rgba(cr: &cairo::Context, c: Rgb, a: f64) {
    cr.set_source_rgba(c.0, c.1, c.2, a);
}

/// x = how exposed, 0..1. Glyph "engaged" level e: filled = the active thing.
fn paint(cr: &cairo::Context, (w, h): (f64, f64), kind: Toggle, x: f64, phase: f64) -> Result<(), cairo::Error> {
    let (cx, cy) = (w / 2.0, h / 2.0);
    let r = w.min(h) / 2.0 - 14.0;
    let col = mix(x);
    let breath = 0.5 + 0.5 * phase.sin();

    let glow = cairo::RadialGradient::new(cx, cy, r * 0.6, cx, cy, r * 1.2);
    glow.add_color_stop_rgba(0.0, col.0, col.1, col.2, 0.18 + 0.22 * x * breath);
    glow.add_color_stop_rgba(1.0, col.0, col.1, col.2, 0.0);
    cr.set_source(&glow)?;
    cr.arc(cx, cy, r * 1.2, 0.0, TAU);
    cr.fill()?;

    cr.set_source_rgb(DISC.0, DISC.1, DISC.2);
    cr.arc(cx, cy, r, 0.0, TAU);
    cr.fill()?;

    let gap = x * 0.95;
    let spin = phase * 0.6;
    rgba(cr, col, 0.95);
    cr.set_line_width(4.0);
    cr.set_line_cap(cairo::LineCap::Round);
    if gap < 0.01 {
        cr.arc(cx, cy, r, 0.0, TAU);
    } else {
        cr.arc(cx, cy, r, spin + gap / 2.0, spin + TAU - gap / 2.0);
    }
    cr.stroke()?;

    let engaged = if kind == Toggle::Firewall { 1.0 - x } else { x };
    cr.save()?;
    cr.translate(cx, cy);
    cr.scale(r * 0.52, r * 0.52);
    cr.set_line_width(0.11);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);
    match kind {
        Toggle::Wine => glass(cr, col, engaged, phase)?,
        Toggle::Xdotool => cursor(cr, col, engaged)?,
        Toggle::Firewall => shield(cr, col, engaged)?,
    }
    cr.restore()
}

fn bowl(cr: &cairo::Context) {
    cr.new_path();
    cr.move_to(-0.55, -0.8);
    cr.curve_to(-0.55, -0.05, -0.3, 0.22, 0.0, 0.22);
    cr.curve_to(0.3, 0.22, 0.55, -0.05, 0.55, -0.8);
    cr.close_path();
}

fn glass(cr: &cairo::Context, col: Rgb, e: f64, phase: f64) -> Result<(), cairo::Error> {
    if e > 0.01 {
        bowl(cr);
        cr.save()?;
        cr.clip();
        let level = 0.22 - 0.75 * e;
        rgba(cr, col, 0.9);
        cr.move_to(-0.7, 1.0);
        for i in 0..=14 {
            let px = -0.7 + i as f64 * 0.1;
            cr.line_to(px, level + 0.035 * (phase * 2.0 + i as f64 * 0.8).sin());
        }
        cr.line_to(0.7, 1.0);
        cr.close_path();
        cr.fill()?;
        cr.restore()?;
    }
    rgba(cr, col, 1.0);
    bowl(cr);
    cr.stroke()?;
    cr.move_to(0.0, 0.22);
    cr.line_to(0.0, 0.72);
    cr.move_to(-0.34, 0.74);
    cr.line_to(0.34, 0.74);
    cr.stroke()
}

fn cursor(cr: &cairo::Context, col: Rgb, e: f64) -> Result<(), cairo::Error> {
    let pts = [(-0.4, -0.75), (-0.4, 0.6), (-0.08, 0.3), (0.13, 0.78), (0.32, 0.7), (0.12, 0.25), (0.52, 0.25)];
    cr.new_path();
    cr.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        cr.line_to(p.0, p.1);
    }
    cr.close_path();
    rgba(cr, col, 0.92 * e);
    cr.fill_preserve()?;
    rgba(cr, col, 1.0);
    cr.stroke()
}

fn shield(cr: &cairo::Context, col: Rgb, e: f64) -> Result<(), cairo::Error> {
    cr.new_path();
    cr.move_to(0.0, -0.8);
    cr.line_to(0.55, -0.58);
    cr.curve_to(0.55, 0.1, 0.32, 0.55, 0.0, 0.8);
    cr.curve_to(-0.32, 0.55, -0.55, 0.1, -0.55, -0.58);
    cr.close_path();
    rgba(cr, col, 0.92 * e);
    cr.fill_preserve()?;
    rgba(cr, col, 1.0);
    cr.stroke()?;
    if e > 0.05 {
        cr.set_source_rgba(DISC.0, DISC.1, DISC.2, e);
        cr.move_to(-0.22, -0.02);
        cr.line_to(-0.04, 0.18);
        cr.line_to(0.26, -0.24);
        cr.stroke()?;
    }
    Ok(())
}
