mod computer;
mod app;
mod tui;

use std::path::{Path, PathBuf};

use app::App;
use computer::Computer;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "assembly/standard.rom")]
    rom_file: PathBuf,
    #[arg(short, long)]
    program_file: Option<PathBuf>,
}

fn main() {
    let _ = env_logger::builder()
        .format_timestamp(None)
        .format_target(true)
        .try_init();

    let cli = Cli::parse();

    let rom_data = read_bytes_from_file(&cli.rom_file);
    let computer = Computer::new().with_rom(rom_data).build().unwrap();

    let mut app = App::new(computer);
    if cli.program_file.is_some() {
        let program = read_bytes_from_file(&cli.program_file.unwrap());
        app.computer.load_program(0x1000, &program);
    }

    app.run();

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