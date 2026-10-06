mod backend;
mod branding;
mod cinnamon;
mod config;
mod fonts;
mod menu;
mod selftest;
mod setup;
mod theme;
mod ui;

use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("org.latch.app")
        .build();
    // First arg = page to open: wine, xdotool, firewall, settings.
    let page = std::env::args().nth(1);
    app.connect_activate(move |app| {
        fonts::load();
        theme::init(config::load().theme);
        ui::window::build(app, page.as_deref()).show_all();
        branding::apply();
        std::thread::spawn(cinnamon::refresh_installed);
    });
    app.run_with_args::<&str>(&[])
}
