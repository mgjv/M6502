use crate::app::App;

use ratatui::crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

pub fn process_events(_terminal: &DefaultTerminal, app: &mut App) -> color_eyre::Result<()> {
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
    Ok(())
}
