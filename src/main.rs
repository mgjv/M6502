mod computer;
mod app;
mod tui;

use app::App;
use computer::Computer;


use std::path::{Path, PathBuf};

use clap::Parser;
use ratatui::DefaultTerminal;
use color_eyre::Result;
use ratatui::crossterm::event::{self, Event};

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

    let terminal = ratatui::init();
    run_app(terminal, App::new(&computer))?;
    ratatui::restore();

    // TODO shut down the computer. We should also do this when
    // there is an error.

    Ok(())
}

fn run_app(mut terminal: DefaultTerminal, app: App) -> Result<()> {
    loop {
        terminal.draw(|f| crate::tui::draw_ui(f, &app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }

            match key.code {
                event::KeyCode::Char('q') => return Ok(()),
                // event::KeyCode::Char('c') => app.computer.toggle_clock(),
                // event::KeyCode::Char('r') => app.computer.reset(),
                // event::KeyCode::Char('s') => app.computer.step(),
                // event::KeyCode::Char('p') => app.computer.toggle_pause(),
                _ => {}
            }
        }
    }
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