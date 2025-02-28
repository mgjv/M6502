mod cpu;
mod memory;
mod clock;

use cpu::CPU;
use memory::Memory;
use clock::{Clock, NormalClock, TickCount};

const DEFAULT_MEMORY_SIZE: usize = 0x4000;
const DEFAULT_CLOCK_SPEED: u32 = 1_000_000; // 1 MHz

#[derive(Debug)]
pub struct Computer<T: Clock> {
    cpu: CPU,
    memory: Memory,
    clock: T,
}

impl Computer<NormalClock> {
    pub fn new() -> Self {
        Self { 
            cpu: CPU::new(),
            memory: Memory::new(DEFAULT_MEMORY_SIZE),
            clock: NormalClock::new(DEFAULT_CLOCK_SPEED),
        }
    }
}

impl<T: Clock> Computer<T> {

    pub fn startup_message(&self) -> String {
        format!("6502 emulator - {} bytes memory", self.memory.size())
    }

    pub fn run(&mut self) {
        let mut number_of_ticks: TickCount = 1; 
        loop {
            self.clock.tick(number_of_ticks);
            match self.cpu.fetch_and_execute(&self.memory) {
                Some(n) => number_of_ticks = n,
                None => break,
            }
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        let mut address = 0;
        for b in program {
            self.memory.write_byte(address, *b);
            address += 1;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction() {
        let computer = Computer {
            cpu: CPU::new(),
            memory: Memory::new(0x10000),
            clock: NormalClock::new(1_000),
        };
        print!("{}", computer.startup_message());

        let computer = Computer {
            cpu: CPU::new(),
            memory: Memory::new(0x100),
            clock: NormalClock::new(1_000),
        };
        print!("{}", computer.startup_message());
    }

    #[test]
    fn load_program() {
        let program = vec![0xa9, 0x01, 0x69, 0x02, 0x8d, 0x02];
        let mut computer = Computer {
            cpu: CPU::new(),
            memory: Memory::new(0x0100),
            clock: NormalClock::new(1_000),
        };

        computer.load_program(&program);

        for i in 0..program.len() {
            let data = computer.memory.read_byte(i as u16);
            assert_eq!(program[i], data);
        }
    }
}