mod events;
mod widgets;

use ratatui::style::{Color, Modifier, Style};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding};
use ratatui::{DefaultTerminal, Frame};

use crate::app::App;

use widgets::address::AddressWidget;
use widgets::memory::MemoryWidget;
use widgets::register::RegisterWidget;
use widgets::status_register::StatusRegisterWidget;

pub fn run_app(app: App)  -> color_eyre::Result<()> {
    let terminal = ratatui::init();
    let result = event_loop(terminal, app);
    // Ensure we clean up when we exit or in case of an error
    ratatui::restore();
    result
}

fn event_loop(mut terminal: DefaultTerminal, mut app: App) -> color_eyre::Result<()> {

    while !app.should_quit {
        app.update();
        terminal.draw(|f| draw_ui(f, &app))?;
        events::process_events(&terminal, &mut app)?;
    }
    Ok(())
}

pub fn draw_ui(frame: &mut Frame, app: &App) {

    let frame_chunks = Layout::vertical([
            Constraint::Length(3), // title and menu
            Constraint::Min(1), // central area
            Constraint::Length(1), // status and info bar
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

    let centre_chunks = Layout::horizontal([
            Constraint::Length(20), // left bar
            Constraint::Min(1), // central area
        ])
        .split(frame_chunks[1]);

    let left = Block::bordered()
        .padding(Padding::uniform(1))
        .title(" Cpu ");
    let left_area = left.inner(centre_chunks[0]);

    frame.render_widget(left, centre_chunks[0]);

    let left_chunks = Layout::vertical([
            Constraint::Length(3), // status register
            Constraint::Length(3), // A, X and Y
            Constraint::Length(3), // Stack Pointer, Program Counter
        ])
        .split(left_area);

    let status = StatusRegisterWidget::new(app.cpu_state.status);
    frame.render_widget(status, left_chunks[0]);

    let register_chunks = Layout::horizontal([
            Constraint::Length(4), // Accumulator
            Constraint::Length(4), // X index
            Constraint::Length(4), // Y index
        ])
        .split(left_chunks[1]);

    frame.render_widget(RegisterWidget::new("A".to_string(), app.cpu_state.accumulator), register_chunks[0]);
    frame.render_widget(RegisterWidget::new("X".to_string(), app.cpu_state.x_index), register_chunks[1]);
    frame.render_widget(RegisterWidget::new("Y".to_string(), app.cpu_state.y_index), register_chunks[2]);

    let sp_and_pc_chunks = Layout::horizontal([
            Constraint::Length(4), // Stack Pointer
            Constraint::Length(6), // Program Counter
        ])
        .split(left_chunks[2]);

    frame.render_widget(RegisterWidget::new("SP".to_string(), app.cpu_state.stack_pointer), sp_and_pc_chunks[0]);
    frame.render_widget(AddressWidget::new("PC".to_string(), app.cpu_state.program_counter), sp_and_pc_chunks[1]);

    let right = Block::new().title(" Memory ");
    let memory_area = right.inner(centre_chunks[1]);
    frame.render_widget(right, centre_chunks[1]);

    let program_counter = app.cpu_state.program_counter;
    let memory_widget = MemoryWidget::new(app, program_counter - 16).set_focus(program_counter);
    frame.render_widget(memory_widget, memory_area);

}
