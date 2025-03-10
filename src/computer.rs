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
        writeln!(b, "Stack:");
        self.cpu.show_stack(&mut b);

        return b
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;

    use test_log::test;

    use super::*;

    fn setup() -> Computer<NormalClock> {
        let rom_file_name = Path::new("assembly/basic.rom");
        let rom = std::fs::read(rom_file_name).expect(
            format!("Was not able to load rom from {}", rom_file_name.display()).as_str()
        );
        Computer::new(&rom)
    }

    fn read_program(file_name: &str) -> Vec<u8> {
        let program_file_name = Path::new(file_name);
        std::fs::read(program_file_name).expect(
            format!("Was not able to load program from {}", program_file_name.display()).as_str()
        )
    }

    #[test]
    fn construction() {
        let computer = setup();
        print!("{}", computer.startup_message());
    }
    
    #[test]
    fn run_lda_test() {
        let mut computer = setup();
        let program = read_program("assembly/lda.test");
        let start_address = 0x1000;
        computer.load_program(start_address, &program);
        computer.run();
        // print!("{}", computer.show_state());
    }
}