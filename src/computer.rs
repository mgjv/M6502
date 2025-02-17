mod cpu;
mod memory;

use cpu::CPU;
use memory::Memory;

const DEFAULT_MEMORY_SIZE: usize = 0x4000;
const DEFAULT_START_ADDRESS: u16 = 0x1000;

pub struct Computer {
    cpu: CPU,
    memory: Memory,
}

impl Default for Computer {
    fn default() -> Self {
        Self { 
            cpu: CPU::new(DEFAULT_START_ADDRESS),
            memory: Memory::new(DEFAULT_MEMORY_SIZE),
        }
    }
}

impl Computer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn startup_message(&self) -> String {
        format!("6502 emulator - {} bytes memory", self.memory.size())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computer_test() {
        let computer = Computer{
            cpu: CPU::new(0x0001),
            memory: Memory::new(0x10000),
        };
    }
}