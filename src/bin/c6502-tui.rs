use m6502::tui::{self, app::App};
use m6502::binutils::{build_computer, Cli};

use clap::Parser;
use color_eyre::Result;
use ratatui::crossterm::event::{self, Event};

fn main() -> Result<()> {
    color_eyre::install()?;
    // TODO: We need to direct the logs somewhere useful
    // TODO: See the tui-logger crate
    let _ = env_logger::builder()
        .format_timestamp(None)
        .format_target(true)
        .try_init();

    let cli = Cli::parse();

    let computer = build_computer(cli.clone());
    // TODO Start the computer in a separate thread, with the correct
    // communication stuff done

    let app = App::new(&computer);

    let terminal = ratatui::init();
    let result = event_loop(terminal, app);
    // Ensure we clean up when we exit or in case of an error
    ratatui::restore();

    // TODO shut down the computer.

    result
}

fn event_loop(mut terminal: ratatui::DefaultTerminal, mut app: App) -> color_eyre::Result<()> {
    while !app.should_quit {

        // Update the internal state of the App
        app.proxy.update();

        // Draw the terminal, based on thaty state
        terminal.draw(|f| tui::draw_tui(f, &app))?;

        // Process any external events, for the next iteration
        if let Event::Key(key) = event::read()? {
            if key.kind != event::KeyEventKind::Release {
                match key.code {
                    event::KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    event::KeyCode::Left => {
                        todo!();
                    }
                    event::KeyCode::Right => {
                        todo!();
                    }
                    // event::KeyCode::Char('c') => app.computer.toggle_clock(),
                    // event::KeyCode::Char('r') => app.computer.reset(),
                    // event::KeyCode::Char('s') => app.computer.step(),
                    // event::KeyCode::Char('p') => app.computer.toggle_pause(),
                    _ => {}
                }
            }
        }
    }

    Ok(())
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