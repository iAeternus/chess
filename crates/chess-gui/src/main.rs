mod app;
mod constants;
mod game;
mod gui;

use crate::app::App;

fn main() {
    let mut app = App::new();
    if let Err(err) = app.run() {
        eprintln!("App error: {err}");
    }
}
