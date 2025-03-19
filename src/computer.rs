mod cpu;
mod bus;
pub mod clock;

use cpu::Cpu;
use bus::{Bus, Ram};
use clock::{Clock, ClockTrait, TickCount};
use log::info;

use std::{fmt::Write, path::PathBuf};

const DEFAULT_CLOCK_SPEED: u32 = 1_000_000; // 1 MHz

pub struct ComputerBuilder {
    clock: Clock,
    rom: Vec<u8>,
    memory_size: usize,
}

impl Default for ComputerBuilder {
    fn default() -> Self {
        Self {
            clock: Clock::default(),
            rom: Vec::new(),
            memory_size: 0x10000,
        }
    }
}

#[allow(dead_code)]
impl ComputerBuilder {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    pub fn with_rom(mut self, rom: Vec<u8>) -> Self {
        self.rom = rom;
        self
    }

    pub fn with_rom_from_file(mut self, file_name: PathBuf) -> Result<Self, std::io::Error> {
        self.rom = std::fs::read(file_name)?;
        Ok(self)
    }

    pub fn with_memory_size(mut self, memory_size: usize) -> Self {
        self.memory_size = memory_size;
        self
    }

    pub fn build(self) -> Result<Computer, String> {
        // Do some sanity checks
        if self.rom.len() < 0x100 {
            return Err("ROM is too small or not set".to_string());
        }

        // Create memory
        let memory = Ram::new(self.memory_size);

        // Build the bus
        let bus = Bus::new()
            .add_ram(memory, 0x0)?
            .add_rom_at_end(&self.rom)?;

        // Build the Cpu
        let cpu = Cpu::new(bus);

        info!("Adding cpu: {}", "6502");
        info!("Adding clock: {}", self.clock);

        // and build the computer
        let mut computer = Computer {
            cpu,
            clock: self.clock,
        };

        // TODO This is needed to run the ROM initialisation. Can be removed in the future
        computer.run();

        Ok(computer)
    }
}

#[derive(Debug)]
pub struct Computer {
    cpu: Cpu,
    //bus: Rc<dyn Addressable>,
    clock: Clock,
}

impl Computer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ComputerBuilder {
        ComputerBuilder::new()
    }
}

impl Computer {
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

    #[allow(unused_must_use)]
    pub fn show_state(&self) -> String {
        let mut buffer = String::new();

        // Let's show the program, memory
        writeln!(buffer, "Registers:\tStatus:");
        self.cpu.show_registers(&mut buffer);
        writeln!(buffer, "Program memory (PC location in red):");
        self.cpu.show_program_memory(&mut buffer);
        writeln!(buffer, "Reset memory:");
        self.cpu.show_reset_memory(&mut buffer);
        writeln!(buffer, "Stack:");
        self.cpu.show_stack(&mut buffer);

        buffer
    }
}


#[cfg(test)]
mod tests {
    use std::{path::Path, process::{Command, Stdio}};
    use log::{debug, info};

    use test_log::test;
    use test_case::test_case;
    use std::sync::Once;

    use clock::SpeedyClock;

    static MAKE_ASSEMBLY: Once = Once::new();

    use super::*;

    #[test]
    fn construction() {
        create_test_computer();
    }

    #[test_case("framework"; "test framework")]
    #[test_case("jump"; "jump and return")]
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
    #[test_case("other"; "other instructions")]
    fn assembly(test_name: &str) {
        let _ = env_logger::builder()
            .is_test(true)
            .format_timestamp(None)
            .format_target(false)
            .try_init();
        let mut computer = create_test_computer();
        let file_name = format!("assembly/{}.test", test_name);
        let program = read_program(file_name.as_str());
        // NOTE: See assembly/test.cfg
        let start_address = 0x1000;
        debug!("Loading Assembly test {}", file_name);
        computer.load_program(start_address, &program);
        computer.run();
    }

    // HELPERS

    fn create_test_computer() -> Computer {
        MAKE_ASSEMBLY.call_once(build_assembly);
        let rom_file_name = Path::new("assembly/standard.rom");
        let rom = std::fs::read(rom_file_name).unwrap_or_else(|_| panic!(
            "Was not able to load rom from {}", rom_file_name.display()
        ));
        Computer::new()
            .with_rom(rom)
            .with_clock(Clock::Speedy(SpeedyClock::default()))
            .build()
            .unwrap_or_else(|_| panic!("Was not able to create computer"))
    }

    fn read_program(file_name: &str) -> Vec<u8> {
        MAKE_ASSEMBLY.call_once(build_assembly);
        let program_file_name = Path::new(file_name);
        std::fs::read(program_file_name).unwrap_or_else(|_| panic!(
            "Was not able to load program from {}", program_file_name.display()
        ))
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