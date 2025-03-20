use crate::App;

use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::Block;

pub fn draw_ui(frame: &mut Frame, app: &App) {

    let frame_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Top: Menu area
    let title = Line::from(format!(" {} - {} ", app.title, app.version))
        .style(Style::default().fg(Color::Yellow))
        .centered();
    let top = Block::bordered().title(title);
    frame.render_widget(top, frame_chunks[0]);

    // Bottom: status and hint area
    let bottom = Block::new()
        .title(Line::from(" status ").right_aligned())
        .title(Line::from(" hint ").left_aligned())
        .title(Line::from(" message ").centered())
        .style(Style::default().add_modifier(Modifier::REVERSED))
        ;
    frame.render_widget(bottom, frame_chunks[2]);

    let centre_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(16),
            Constraint::Min(1),
        ])
        .split(frame_chunks[1]);

    let left = Block::bordered().title(" Cpu ");
    frame.render_widget(left, centre_chunks[0]);
    let right = Block::new().title(" Memory ");
    frame.render_widget(right, centre_chunks[1]);
}