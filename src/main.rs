mod computer;

use std::path::PathBuf;

use computer::Computer;
use computer::clock::NormalClock;
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

fn append_to_path(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    let map = cli.rom_map.unwrap_or_else(|| append_to_path(cli.rom_file.clone(), ".map"));

    println!("Map: {}", map.display());

    let rom_data = read_bytes_from_file(cli.rom_file);
    let mut computer = Computer::new(&rom_data, NormalClock::default());
    show_debug(&computer.startup_message());

    show_debug(&computer.show_state());

    let program = read_bytes_from_file(cli.program_file);
    computer.load_program(0x1000, &program);

    // dbg!(&computer);
    show_debug(&computer.show_state());

    computer.run();

    show_debug(&computer.show_state());
}

fn read_bytes_from_file(file: PathBuf) -> Vec<u8> {
    let file_name = file.as_path();
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