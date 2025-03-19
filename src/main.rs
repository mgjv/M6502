mod computer;
mod app;
mod tui;

use std::path::{Path, PathBuf};

use computer::Computer;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    // Name of the ROM to load. Mandatory argument
    rom_file: PathBuf,

    program_file: PathBuf,

    // Sets a custom map file for the ROM. Otherwise it will be derived from the ROM name
    #[arg(short, long("map"), value_name = "ROM MAP FILE")]
    rom_map: Option<PathBuf>,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    let map = cli.rom_map.unwrap_or_else(|| {
        let mut map = cli.rom_file.clone();
        map.push(".map");
        map
    });
    println!("Map: {}", map.display());

    run_computer(&cli.rom_file.as_path(), cli.program_file.as_path());

}

fn run_computer(rom_file: &Path, program_file: &Path) {
    let rom_data = read_bytes_from_file(rom_file);
    let mut computer = Computer::new().with_rom(rom_data).build().unwrap();
    show_debug(&computer.startup_message());

    show_debug(&computer.show_state());

    let program = read_bytes_from_file(program_file);
    computer.load_program(0x1000, &program);

    // dbg!(&computer);
    show_debug(&computer.show_state());

    computer.run();

    show_debug(&computer.show_state());
}

fn read_bytes_from_file(file_name: &Path) -> Vec<u8> {
    std::fs::read(file_name).unwrap_or_else(|_| panic!(
        "Was not able to load bytes from {}", file_name.display()
    ))
}

fn show_debug(s: &str) {
    if log::max_level() >= log::LevelFilter::Debug {
        println!("{}", s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}