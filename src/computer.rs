mod cpu;
mod memory;
mod clock;

use cpu::CPU;
use memory::Memory;
use clock::{Clock, NormalClock, TickCount};

use std::fmt::Write;

const DEFAULT_MEMORY_SIZE: usize = 0x4000;
const DEFAULT_CLOCK_SPEED: u32 = 1_000_000; // 1 MHz

#[derive(Debug)]
pub struct Computer<C: Clock> {
    cpu: CPU<Memory>,
    clock: C,
}

impl Computer<NormalClock> {
    pub fn new() -> Self {
    Self { 
            cpu: CPU::new(Memory::new(DEFAULT_MEMORY_SIZE)),
            clock: NormalClock::new(DEFAULT_CLOCK_SPEED),
        }
    }
}

impl<C: Clock> Computer<C> {


    pub fn run(&mut self) {
        let mut number_of_ticks: TickCount = 1; 
        loop {
            self.clock.tick(number_of_ticks);
            match self.cpu.fetch_and_execute() {
                Some(n) => number_of_ticks = n,
                None => break,
            }
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.load_program(program);
    }
}

// Formatting/Display functions
impl<C: Clock> Computer<C> {

    pub fn startup_message(&self) -> String {
        format!("6502 emulator - {} bytes memory", self.cpu.memory_size())
    }

    #[allow(unused_must_use)]
    pub fn show_state(&self) -> String {
        let mut b = String::new();

        // Let's show the program, memory
        writeln!(b, "Registers:\tStatus:");
        self.cpu.show_registers(&mut b);
        writeln!(b, "Program memory:");
        self.cpu.show_program_memory(&mut b);

        b
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction() {
        let computer = Computer {
            cpu: CPU::new(Memory::new(0x10000)),
            clock: NormalClock::new(1_000),
        };
        print!("{}", computer.startup_message());

        let computer = Computer {
            cpu: CPU::new(Memory::new(0x100)),
            clock: NormalClock::new(1_000),
        };
        print!("{}", computer.startup_message());
    }
}