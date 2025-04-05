use crate::computer::{cpu::inspect::CpuState, Computer};

// App contains the model functionality for any UI to display
// the state of a computer
pub struct App<'a> {
    // Some display stuff
    pub title: String,
    pub version: String,

    // A private reference to the computer we're shadowing
    computer: &'a Computer,

    // The current state of the CPU. Refresh with self.update()
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

    // Update the state from the computer
    pub fn update(&mut self) {
        self.cpu_state = self.computer.get_cpu_state();
    }

    // Get memory contents from the computer's bus.
    pub fn get_memory_lines(&self, address: u16, line_count: u16, line_length: u16) -> Vec<(u16, Vec<u8>)> {
        // start needs to be aligned with line_length, and address should be on the second line
        // TODO guard against dropping below 0?
        let start = address - address % line_length - line_length;
        self.computer.get_memory_lines(start, line_count, line_length)
    }

    pub fn current_opcode_to_string(&self) -> String {
        self.computer.address_opcode_to_string(self.cpu_state.program_counter)
    }

    pub fn disassemble(&self, start_address: u16, length: u16) -> Vec<(u16, String)> {
        self.computer.disassemble(start_address, length)
    }

    pub fn get_execution_history(&self) -> Vec<(u16, String)> {
        self.computer.get_execution_history()
    }

    pub fn get_execution_future(&self) -> Vec<(u16, String)> {
        self.computer.disassemble(self.cpu_state.program_counter, 16)
    }

}