mod widgets;
pub mod app;

use app::App;
use ratatui::prelude::*;
use ratatui::widgets::*;
use widgets::*;

// const NORMAL_STYLE: Style = Style::new().fg(Color::White).bg(Color::Black);
const BLOCK_TITLE_STYLE: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
const APP_TITLE_STYLE: Style = Style::new()
    .fg(Color::Blue)
    .bg(Color::Yellow)
    .add_modifier(Modifier::BOLD.union(Modifier::ITALIC));
const STATUS_BAR_STYLE: Style = Style::new().fg(Color::Black).bg(Color::White);
const SELECTED_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Yellow);
const BLOCK_PADDING: Padding = Padding::horizontal(1);
const PAD_SPACE_V: u16 = BLOCK_PADDING.top + BLOCK_PADDING.bottom;

pub fn draw_tui(frame: &mut Frame, app: &App) {
    let [top, middle, bottom] = Layout::vertical([
        Constraint::Length(3), // title and menu
        Constraint::Min(1),    // central area
        Constraint::Length(1), // status and info bar
    ])
    .areas(frame.area());

    draw_top_area(top, frame, app);

    let [left_middle, right_middle] = Layout::horizontal([
        Constraint::Length(28), // left bar
        Constraint::Min(1),     // central area
    ])
    .areas(middle);

    draw_left_bar(left_middle, frame, app);
    draw_memory_area(right_middle, frame, app);

    draw_status_bar(bottom, frame, app);
}

fn draw_left_bar(area: Rect, frame: &mut Frame, app: &App) {
    let [cpu_area, execution_area] = Layout::vertical([
        Constraint::Length(5 + PAD_SPACE_V),
        Constraint::Fill(1),
    ])
    .areas(area);

    draw_cpu_monitor(cpu_area, frame, app);
    draw_execution(execution_area, frame, app);
}

fn draw_cpu_monitor(area: Rect, frame: &mut Frame, app: &App) {
    let left = Block::bordered()
        .padding(BLOCK_PADDING)
        .title(" Cpu ")
        .title_style(BLOCK_TITLE_STYLE);
    let left_area = left.inner(area);

    frame.render_widget(left, area);

    let [program_counter_area, register_area, status_register_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .areas(left_area);

    let [pc_area, op_area] = Layout::horizontal([
        Constraint::Min(7), // Program Counter
        Constraint::Min(1), // Operation
    ])
    .areas(program_counter_area);

    frame.render_widget(
        AddressWidget::new("PC".to_string(), app.proxy.cpu_state.program_counter),
        pc_area,
    );
    frame.render_widget(
        Text::raw(app.proxy.current_opcode_to_string()),
        op_area,
    );

    let [a_area, x_area, y_area, sp_area] = Layout::horizontal([
        Constraint::Min(5), // Accumulator
        Constraint::Min(5), // X index
        Constraint::Min(5), // Y index
        Constraint::Min(5), // Stack Pointer
    ])
    .areas(register_area);

    frame.render_widget(
        RegisterWidget::new("A".to_string(), app.proxy.cpu_state.accumulator),
        a_area,
    );
    frame.render_widget(
        RegisterWidget::new("X".to_string(), app.proxy.cpu_state.x_index),
        x_area,
    );
    frame.render_widget(
        RegisterWidget::new("Y".to_string(), app.proxy.cpu_state.y_index),
        y_area,
    );
    frame.render_widget(
        RegisterWidget::new("SP".to_string(), app.proxy.cpu_state.stack_pointer),
        sp_area,
    );

    let status = StatusRegisterWidget::new(app.proxy.cpu_state.status);
    frame.render_widget(status, status_register_area);
}

fn draw_execution(area: Rect, frame: &mut Frame, app: &App) {
    let right = Block::bordered()
        .padding(BLOCK_PADDING)
        .title(" Program assembly ")
        .title_style(BLOCK_TITLE_STYLE);
    let right_area = right.inner(area);
    frame.render_widget(right, area);

    let history = app.proxy.get_execution_history();

    let all_items: Vec<String> = history
        .iter()
        .chain(app.proxy.get_execution_future().iter())
        .map(|x| format!("{:04x}: {}", x.0, x.1))
        .collect();

    let mut state = ListState::default();
    state.select(Some(history.len()));

    let list = List::new(all_items)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">");

    frame.render_stateful_widget(list, right_area, &mut state);
}

fn draw_memory_area(area: Rect, frame: &mut Frame, app: &App) {
    let right = Block::bordered()
        .title(" Memory ")
        .padding(Padding::uniform(1))
        .title_style(BLOCK_TITLE_STYLE);
    let memory_area = right.inner(area);
    frame.render_widget(right, area);

    let program_counter = app.proxy.cpu_state.program_counter;
    let memory_widget = MemoryWidget::new(app, program_counter - 16).set_focus(program_counter);
    frame.render_widget(memory_widget, memory_area);
}

fn draw_top_area(area: Rect, frame: &mut Frame, app: &App) {
    // Top: Menu area
    let top = Block::bordered()
        .title(format!(" {} - {} ", app.title, app.version))
        .title_style(APP_TITLE_STYLE)
        .title_alignment(Alignment::Center);

    frame.render_widget(top, area);
}

fn draw_status_bar(area: Rect, frame: &mut Frame, _app: &App) {
    // Bottom: status and hint area
    let bottom = Block::new()
        .title(Line::from(" status ").right_aligned())
        .title(Line::from(" hint ").left_aligned())
        .title(Line::from(" message ").centered())
        .style(STATUS_BAR_STYLE);

    frame.render_widget(bottom, area);
}
