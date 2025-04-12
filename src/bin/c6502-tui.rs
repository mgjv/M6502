use m6502::tui::App;
use m6502::binutils::{build_computer, Cli};

use clap::Parser;
use color_eyre::Result;

fn main() -> Result<()> {
    // Some setup
    tui_logger::init_logger(log::LevelFilter::Trace)?;
    tui_logger::set_default_level(log::LevelFilter::Trace);
    color_eyre::install()?;

    let cli = Cli::parse();

    let computer = build_computer(cli);
    // TODO Start the computer in a separate thread, with the correct
    // communication stuff done

    let terminal = ratatui::init();
    let result = App::new(&computer).run(terminal);
    // Ensure we clean up when we exit or in case of an error
    ratatui::restore();

    // TODO shut down the computer.

    result
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