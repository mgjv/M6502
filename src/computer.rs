mod cpu;
mod memory;
pub mod clock;

use cpu::CPU;
use memory::Memory;
use clock::*;

use std::fmt::Write;

// TODO Work on memory mapping in memory.rs to allow smaller memory
// while still providing the needed vectors at the end of memory space
const DEFAULT_CLOCK_SPEED: u32 = 1_000_000; // 1 MHz

#[derive(Debug)]
pub struct Computer<C: Clock> {
    cpu: CPU<Memory>,
    clock: C,
}

impl<C: Clock> Computer<C> {
    pub fn new(rom_data: &[u8], clock: C) -> Self {

        let cpu = CPU::new(Memory::new(), rom_data);

        let mut new_computer = Self {
            cpu: cpu,
            clock: clock,
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

    // Formatting/Display functions

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
    use std::{path::Path, process::{Command, Stdio}};
    use log::{debug, info};

    use test_log::test;
    use test_case::test_case;
    use std::sync::Once;

    static MAKE_ASSEMBLY: Once = Once::new();

    use super::*;

    #[test]
    fn construction() {
        let computer = create_test_computer();
        print!("{}", computer.startup_message());
    }

    // TODO write a test to check that the test cases are there
    // maybe run make automatically?

    #[test_case("framework"; "test framework")]
    #[test_case("flags"; "status flags")]
    #[test_case("branches"; "conditional branches")]
    #[test_case("address_modes"; "address modes")]
    #[test_case("transfer"; "transfer instructions")]
    #[test_case("stack"; "stack operation")]
    #[test_case("increment"; "increment and decrement")]
    #[test_case("logical"; "logical instructions")]
    #[test_case("bitshift"; "bit shift insgtructions")]
    #[test_case("add_with_carry"; "add with carry")]
    #[test_case("comparison"; "comparison instructions")]
    fn assembly(test_name: &str) {
        let _ = env_logger::builder()
            .is_test(true)
            .format_timestamp(None)
            .format_target(false)
            .try_init();
        let mut computer = create_test_computer();
        let file_name = format!("assembly/{}.test.bin", test_name);
        let program = read_program(file_name.as_str());
        // NOTE: See assembly/test.cfg
        let start_address = 0x1000;
        debug!("Loading Assembly test {}", file_name);
        computer.load_program(start_address, &program);
        computer.run();
    }

    // HELPERS

    fn create_test_computer() -> Computer<SpeedyClock> {
        MAKE_ASSEMBLY.call_once(build_assembly);
        let rom_file_name = Path::new("assembly/basic.rom");
        let rom = std::fs::read(rom_file_name).expect(
            format!("Was not able to load rom from {}", rom_file_name.display()).as_str()
        );
        Computer::new(&rom, SpeedyClock {})
    }

    fn read_program(file_name: &str) -> Vec<u8> {
        MAKE_ASSEMBLY.call_once(build_assembly);
        let program_file_name = Path::new(file_name);
        std::fs::read(program_file_name).expect(
            format!("Was not able to load program from {}", program_file_name.display()).as_str()
        )
    }

    fn build_assembly() {
        info!("Building assembly");
        let status = Command::new("make")
            .arg("-C").arg("assembly") // chdir to assembly
            .arg("-j").arg("8") // run this many in parallel
            .stdout(Stdio::null())
            .status()
            .expect("Make failed to run");

        assert!(status.success(), "Make returned an error. Please run from command line to check")
    }

}