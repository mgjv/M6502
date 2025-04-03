mod events;
mod widgets;

use ratatui::{prelude::*, widgets::*};
use widgets::*;

use crate::app::App;

pub fn run_app(app: App) -> color_eyre::Result<()> {
    let terminal = ratatui::init();
    let result = event_loop(terminal, app);
    // Ensure we clean up when we exit or in case of an error
    ratatui::restore();
    result
}

fn event_loop(mut terminal: ratatui::DefaultTerminal, mut app: App) -> color_eyre::Result<()> {
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
        Constraint::Min(1),    // central area
        Constraint::Length(1), // status and info bar
    ])
    .split(frame.area());

    draw_top_bar(frame_chunks[0], frame, app);

    let main_chunks = Layout::horizontal([
        Constraint::Length(28), // left bar
        Constraint::Min(1),     // central area
    ])
    .split(frame_chunks[1]);

    draw_left_bar(main_chunks[0], frame, app);
    draw_memory_area(main_chunks[1], frame, app);

    draw_status_bar(frame_chunks[2], frame, app);
}

fn draw_left_bar(area: Rect, frame: &mut Frame, app: &App) {
    let left_chunks = Layout::vertical([
        Constraint::Length(5), // cpu monitor
        Constraint::Fill(1),   // rest
    ])
    .split(area);

    draw_cpu_monitor(left_chunks[0], frame, app);
}

fn draw_cpu_monitor(area: Rect, frame: &mut Frame, app: &App) {
    let left = Block::bordered()
        //.padding(Padding::uniform(1))
        .title(" Cpu ")
        .title_style(Style::new().blue().on_white().bold());
    let left_area = left.inner(area);

    frame.render_widget(left, area);

    let left_chunks = Layout::vertical([
        Constraint::Length(2), // status register
        Constraint::Length(2), // A, X and Y
        Constraint::Length(2), // Stack Pointer, Program Counter
    ])
    .split(left_area);

    let sp_and_pc_chunks = Layout::horizontal([
        Constraint::Min(7), // Program Counter
        Constraint::Min(1), // Operation
    ])
    .split(left_chunks[0]);

    frame.render_widget(
        AddressWidget::new("PC".to_string(), app.cpu_state.program_counter),
        sp_and_pc_chunks[0],
    );
    frame.render_widget(
        Text::raw(app.current_opcode_to_string()),
        sp_and_pc_chunks[1],
    );

    let register_chunks = Layout::horizontal([
        Constraint::Min(5), // Accumulator
        Constraint::Min(5), // X index
        Constraint::Min(5), // Y index
        Constraint::Min(5), // Stack Pointer
    ])
    .split(left_chunks[1]);

    frame.render_widget(
        RegisterWidget::new("A".to_string(), app.cpu_state.accumulator),
        register_chunks[0],
    );
    frame.render_widget(
        RegisterWidget::new("X".to_string(), app.cpu_state.x_index),
        register_chunks[1],
    );
    frame.render_widget(
        RegisterWidget::new("Y".to_string(), app.cpu_state.y_index),
        register_chunks[2],
    );
    frame.render_widget(
        RegisterWidget::new("SP".to_string(), app.cpu_state.stack_pointer),
        register_chunks[3],
    );

    let status = StatusRegisterWidget::new(app.cpu_state.status);
    frame.render_widget(status, left_chunks[2]);
}

fn draw_memory_area(area: Rect, frame: &mut Frame, app: &App) {
    let right = Block::bordered().title(" Memory ");
    let memory_area = right.inner(area);
    frame.render_widget(right, area);

    let program_counter = app.cpu_state.program_counter;
    let memory_widget = MemoryWidget::new(app, program_counter - 16).set_focus(program_counter);
    frame.render_widget(memory_widget, memory_area);
}

fn draw_top_bar(area: Rect, frame: &mut Frame, app: &App) {
    // Top: Menu area
    let title = Line::from(format!(" {} - {} ", app.title, app.version))
        .style(Style::default().fg(Color::Yellow))
        .centered();
    let top = Block::bordered().title(title);
    frame.render_widget(top, area);
}

fn draw_status_bar(area: Rect, frame: &mut Frame, _app: &App) {
    // Bottom: status and hint area
    let bottom = Block::new()
        .title(Line::from(" status ").right_aligned())
        .title(Line::from(" hint ").left_aligned())
        .title(Line::from(" message ").centered())
        .style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(bottom, area);
}
