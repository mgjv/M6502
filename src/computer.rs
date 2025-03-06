mod cpu;
mod memory;
mod clock;

use cpu::CPU;
use memory::Memory;
use clock::{Clock, NormalClock, TickCount};

use std::fmt::Write;

// TODO Work on memory mapping in memory.rs to allow smaller memory
// while still providing the needed vectors at the end of memory space
const DEFAULT_CLOCK_SPEED: u32 = 1_000_000; // 1 MHz

#[derive(Debug)]
pub struct Computer<C: Clock> {
    cpu: CPU<Memory>,
    clock: C,
}

impl Computer<NormalClock> {
    pub fn new(rom_data: &[u8]) -> Self {

        let cpu = CPU::new(Memory::new(), rom_data);

        let mut new_computer = Self { 
            cpu: cpu,
            clock: NormalClock::new(DEFAULT_CLOCK_SPEED),
        };

        // Run the computer, until interrupt
        new_computer.run();

        return new_computer
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

    pub fn load_program(&mut self, address: u16, program: &[u8]) {
        self.cpu.load_program(address, program);
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
        writeln!(b, "Program memory (PC location in red):");
        self.cpu.show_program_memory(&mut b);
        writeln!(b, "Reset memory:");
        self.cpu.show_reset_memory(&mut b);

        return b
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // FIXME This requires the tests module to be public. Not great.
    use cpu::tests::TEST_ROM;

    #[test]
    fn construction() {
        let computer = Computer {
            cpu: CPU::new(Memory::new(), TEST_ROM),
            clock: NormalClock::new(1_000),
        };
        print!("{}", computer.startup_message());

        let computer = Computer {
            cpu: CPU::new(Memory::new(), TEST_ROM),
            clock: NormalClock::new(1_000),
        };
        print!("{}", computer.startup_message());
    }
}