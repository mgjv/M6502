// common utilities for binaries

use std::path::{Path, PathBuf};

use clap::Parser;

use crate::computer::Computer;

pub fn read_bytes_from_file(file_name: &Path) -> Vec<u8> {
    std::fs::read(file_name).unwrap_or_else(|_| panic!(
        "Was not able to load bytes from {}", file_name.display()
    ))
}

// Command line parser for all binaries
#[derive(Parser, Clone)]
pub struct Cli {
    #[arg(short, long, default_value = "assembly/standard.rom")]
    pub rom_file: PathBuf,
    #[arg(short, long)]
    pub program_file: Option<PathBuf>,
}

// Default way to build a computer from command line arguments
pub fn build_computer(cli: Cli) -> Computer {
    let rom_data = read_bytes_from_file(&cli.rom_file);
    let mut computer = Computer::new().with_rom(rom_data).build().unwrap();

    if cli.program_file.is_some() {
        let program = read_bytes_from_file(&cli.program_file.unwrap());
        computer.load_program(0x1000, &program);
    }

    computer
}


