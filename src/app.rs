use log::info;

use crate::computer::Computer;

pub struct App {
    // Some display stuff
    pub title: String,

    // Functional stuff
    pub computer: Computer,
}

impl App {
    pub fn new(computer: Computer) -> Self {
        Self {
            computer,
            title: "CMOS 6502 emulator".to_string(),
        }
    }

    pub fn run(&mut self) {
        info!("Running {}", self.title);

        self.computer.run();
        self.computer.show_state();
    }
}