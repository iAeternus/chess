use std::error::Error;
use std::sync::mpsc;

use crate::gui::view::ViewEgui;

pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // channels
        let (tx, rx) = mpsc::channel();

        // actors
        let view = ViewEgui::new(tx, rx);

        // thread + tokio runtime
        ViewEgui::run(view)?;

        println!("exiting App::run()");
        Ok(())
    }
}
