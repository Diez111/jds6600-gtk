use gtk4::prelude::*;
use gtk4::Application;

mod app;
mod driver;
mod model;
mod waveform;

const APP_ID: &str = "com.diez111.jds6600";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(app::build_ui);
    app.run();
}
