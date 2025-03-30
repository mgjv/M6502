use m6502::app::App;
use m6502::computer::Computer;
use m6502::tui;

use std::path::{Path, PathBuf};

use clap::Parser;
use color_eyre::Result;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "assembly/standard.rom")]
    rom_file: PathBuf,
    #[arg(short, long)]
    program_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = env_logger::builder()
        .format_timestamp(None)
        .format_target(true)
        .try_init();

    let cli = Cli::parse();

    let computer = build_computer(cli);
    // TODO Start the computer in a separate thread, with the correct
    // communication stuff done

    tui::run_app(App::new(&computer))?;

    // TODO shut down the computer. We should also do this when
    // there is an error.

    Ok(())
}

fn build_computer(cli: Cli) -> Computer {
    let rom_data = read_bytes_from_file(&cli.rom_file);
    let mut computer = Computer::new().with_rom(rom_data).build().unwrap();

    if cli.program_file.is_some() {
        let program = read_bytes_from_file(&cli.program_file.unwrap());
        computer.load_program(0x1000, &program);
    }

    computer
}

fn read_bytes_from_file(file_name: &Path) -> Vec<u8> {
    std::fs::read(file_name).unwrap_or_else(|_| panic!(
        "Was not able to load bytes from {}", file_name.display()
    ))
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