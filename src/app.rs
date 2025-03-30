use crate::computer::{cpu::inspect::CpuState, Computer};

pub struct App<'a> {
    // Some display stuff
    pub title: String,
    pub version: String,

    // A reference to the computer we're supposed to shadow
    pub computer: &'a Computer,

    // Cached data
    pub cpu_state: CpuState,

    // Whether the app should be closed
    pub should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(computer: &'a Computer) -> Self {
        Self {
            computer,
            title: "CMOS 6502 emulator".to_string(),
            version: "0.0.1".to_string(),
            cpu_state: computer.get_cpu_state(),
            should_quit: false,
        }
    }

    pub fn get_memory_lines(&self, address: u16) -> Vec<(u16, Vec<u8>)> {
        let n_lines = 12;
        let line_length = 16;
        // start needs to be aligned with line_length, and address should be on the second line
        // TODO guard against dropping below 0?
        let start = address - address % line_length - line_length;
        self.computer.get_memory_lines(start, n_lines, line_length)
    }
}