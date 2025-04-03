use super::*;
use super::bus::Addressable;
use super::cpu::inspect::CpuState;

impl Computer {
    pub fn get_cpu_state(&self) -> CpuState {
        self.cpu.get_state()
    }

    // Returns a vector of lines representing memory.
    // start has to be aligned with line_length
    pub fn get_memory_lines(&self, start: u16, n_lines: u16, line_length: u16) -> Vec<(u16, Vec<u8>)> {
        assert!(start % line_length == 0);
        let mut lines = Vec::new();
        for i in 0..n_lines {
            let mut line = Vec::new();
            for j in 0..line_length {
                line.push(self.cpu.bus.read_byte(start + i * line_length + j));
            }
            lines.push((start + i * line_length, line));
        }
        lines
    }

    pub fn address_opcode_to_string(&self, address: u16) -> String {
        self.cpu.address_opcode_to_string(address)
    }
}
