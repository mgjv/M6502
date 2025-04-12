mod widgets;

use crossterm::event::KeyEvent;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::*;
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::computer::Computer;
use crate::proxy::ComputerProxy;

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

#[derive(Clone, Copy, Debug)]
enum AppDisplayState {
    MainWindow,
    LogPopup, // true = display timestamp
}

pub struct App<'a> {
    title: String,
    version: String,

    proxy: ComputerProxy<'a>,

    should_quit: bool,

    display_state: AppDisplayState,
    display_log_timestamp: bool,

    // State for widgets
    assembly_list_state: RefCell<ListState>,
    log_widget_state: RefCell<TuiWidgetState>,
}

impl<'a> App<'a> {
    pub fn new(computer: &'a Computer) -> Self {
        Self {
            title: "CMOS 6502 emulator".to_string(),
            version: "0.0.1".to_string(),

            proxy: ComputerProxy::new(computer),

            should_quit: false,

            display_state: AppDisplayState::MainWindow,
            display_log_timestamp: true,

            assembly_list_state: ListState::default().into(),
            log_widget_state: TuiWidgetState::new()
                .set_default_display_level(log::LevelFilter::Debug)
                .into(),
        }
    }

    pub fn run(mut self, mut terminal: ratatui::DefaultTerminal) -> color_eyre::Result<()> {
        while !self.should_quit {
            // Update the internal state of the App
            self.proxy.update();

            // Draw the terminal, based on that state
            terminal.draw(|f| self.draw_tui(f))?;

            // Process any interesting events
            self.process_events()?;
        }
        // If we're here, we're quitting
        Ok(())
    }

    fn process_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()? {
            // Common/global keys
            if key.kind != KeyEventKind::Release {
                match key.code {
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }

            match self.display_state {
                AppDisplayState::MainWindow => {
                    self.process_main_window_event(key);
                }
                AppDisplayState::LogPopup => {
                    self.process_log_popup_event(key);
                }
            }
        }
        // Keys depending on application state
        Ok(())
    }

    fn process_main_window_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Release {
            match key.code {
                KeyCode::Char('l') => {
                    self.display_state = AppDisplayState::LogPopup;
                }
                _ => {}
            }
        }
    }

    fn process_log_popup_event(&mut self, key: KeyEvent) {
        // Process any events in the log popup
        if key.kind != KeyEventKind::Release {
            let state = self.log_widget_state.borrow();
            let state = state.deref();

            match key.code {
                KeyCode::Char('l') => {
                    self.display_state = AppDisplayState::MainWindow;
                }
                KeyCode::Char('t') => {
                    self.display_log_timestamp = !self.display_log_timestamp;
                }
                // See TuiWidgetEvent for the meaning of these keys
                KeyCode::Char(' ') => state.transition(TuiWidgetEvent::SpaceKey),
                KeyCode::Esc => state.transition(TuiWidgetEvent::EscapeKey),
                KeyCode::PageUp => state.transition(TuiWidgetEvent::PrevPageKey),
                KeyCode::PageDown => state.transition(TuiWidgetEvent::NextPageKey),
                KeyCode::Up => state.transition(TuiWidgetEvent::UpKey),
                KeyCode::Down => state.transition(TuiWidgetEvent::DownKey),
                KeyCode::Left => state.transition(TuiWidgetEvent::LeftKey),
                KeyCode::Right => state.transition(TuiWidgetEvent::RightKey),
                KeyCode::Char('+') => state.transition(TuiWidgetEvent::PlusKey),
                KeyCode::Char('-') => state.transition(TuiWidgetEvent::MinusKey),
                KeyCode::Char('h') => state.transition(TuiWidgetEvent::HideKey),
                KeyCode::Char('f') => state.transition(TuiWidgetEvent::FocusKey),
                _ => {}
            }
        }
    }

    fn draw_tui(&self, frame: &mut Frame) {
        let [top, middle, bottom] = Layout::vertical([
            Constraint::Length(3), // title and menu
            Constraint::Min(1),    // central area
            Constraint::Length(1), // status and info bar
        ])
        .areas(frame.area());

        self.draw_top_area(top, frame);

        let [left_middle, right_middle] = Layout::horizontal([
            Constraint::Length(28), // left bar
            Constraint::Min(1),     // central area
        ])
        .areas(middle);

        self.draw_status_bar(bottom, frame);

        match self.display_state {
            AppDisplayState::MainWindow => {
                self.draw_left_bar(left_middle, frame);
                self.draw_memory_area(right_middle, frame);
            }
            AppDisplayState::LogPopup => {
                self.draw_log_popup(frame);
            }
        }
    }

    fn draw_left_bar(&self, area: Rect, frame: &mut Frame) {
        let [cpu_area, execution_area] =
            Layout::vertical([Constraint::Length(5 + PAD_SPACE_V), Constraint::Fill(1)])
                .areas(area);

        self.draw_cpu_monitor(cpu_area, frame);
        self.draw_execution(execution_area, frame);
    }

    fn draw_cpu_monitor(&self, area: Rect, frame: &mut Frame) {
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
            AddressWidget::new("PC".to_string(), self.proxy.cpu_state.program_counter),
            pc_area,
        );
        frame.render_widget(Text::raw(self.proxy.current_opcode_to_string()), op_area);

        self.draw_cpu_registers(register_area, frame);

        let status = StatusRegisterWidget::new(self.proxy.cpu_state.status);
        frame.render_widget(status, status_register_area);
    }

    fn draw_cpu_registers(&self, area: Rect, frame: &mut Frame) {
        let [a_area, x_area, y_area, sp_area] = Layout::horizontal([
            Constraint::Min(5), // Accumulator
            Constraint::Min(5), // X index
            Constraint::Min(5), // Y index
            Constraint::Min(5), // Stack Pointer
        ])
        .areas(area);

        frame.render_widget(
            RegisterWidget::new("A".to_string(), self.proxy.cpu_state.accumulator),
            a_area,
        );
        frame.render_widget(
            RegisterWidget::new("X".to_string(), self.proxy.cpu_state.x_index),
            x_area,
        );
        frame.render_widget(
            RegisterWidget::new("Y".to_string(), self.proxy.cpu_state.y_index),
            y_area,
        );
        frame.render_widget(
            RegisterWidget::new("SP".to_string(), self.proxy.cpu_state.stack_pointer),
            sp_area,
        );
    }

    fn draw_execution(&self, area: Rect, frame: &mut Frame) {
        let right = Block::bordered()
            .padding(BLOCK_PADDING)
            .title(" Program assembly ")
            .title_style(BLOCK_TITLE_STYLE);
        let right_area = right.inner(area);
        frame.render_widget(right, area);

        let history = self.proxy.get_execution_history();

        let all_items: Vec<String> = history
            .iter()
            .chain(self.proxy.get_execution_future().iter())
            .map(|x| format!("{:04x}: {}", x.0, x.1))
            .collect();

        //let mut state = ListState::default();
        //state.select(Some(history.len()));

        let list = List::new(all_items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">");

        let mut state = self.assembly_list_state.borrow_mut();
        frame.render_stateful_widget(list, right_area, state.deref_mut());
    }

    fn draw_memory_area(&self, area: Rect, frame: &mut Frame) {
        let right = Block::bordered()
            .title(" Memory ")
            .padding(Padding::uniform(1))
            .title_style(BLOCK_TITLE_STYLE);
        let memory_area = right.inner(area);
        frame.render_widget(right, area);

        let program_counter = self.proxy.cpu_state.program_counter;
        let memory_widget =
            MemoryWidget::new(self, program_counter - 16).set_focus(program_counter);
        frame.render_widget(memory_widget, memory_area);
    }

    fn draw_top_area(&self, area: Rect, frame: &mut Frame) {
        // Top: Menu area
        let top = Block::bordered()
            .title(format!(" {} - {} ", self.title, self.version))
            .title_style(APP_TITLE_STYLE)
            .title_alignment(Alignment::Center);

        frame.render_widget(top, area);
    }

    fn draw_status_bar(&self, area: Rect, frame: &mut Frame) {
        let message = format!(
            " {} ",
            match self.display_state {
                AppDisplayState::MainWindow => "press l to display log",
                AppDisplayState::LogPopup => "press 'l' to return",
            }
        );
        // Bottom: status and hint area
        let bottom = Block::new()
            .title(Line::from(" status ").right_aligned())
            .title(Line::from(" hint ").left_aligned())
            .title(Line::from(message).centered())
            .style(STATUS_BAR_STYLE);

        frame.render_widget(bottom, area);
    }

    fn draw_log_popup(&self, frame: &mut Frame) {
        let area = self.global_popup_area(frame, 90, 90);
        frame.render_widget(Clear, area);

        let mut state = self.log_widget_state.borrow_mut();
        let state = state.deref_mut();

        let time_stamp = if self.display_log_timestamp {
            Some("%H:%M:%S".to_string())
        } else {
            None
        };

        let smart_log_widget = tui_logger::TuiLoggerSmartWidget::default()
            .title_log(Line::from("Log"))
            .title_target(Line::from("Select log target"))
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Magenta))
            .style_info(Style::default().fg(Color::Cyan))
            .output_timestamp(time_stamp)
            .output_separator(':')
            .output_level(Some(tui_logger::TuiLoggerLevelOutput::Abbreviated))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .state(state);

        frame.render_widget(smart_log_widget, area);
    }

    fn global_popup_area(&self, frame: &Frame, percent_x: u16, percent_y: u16) -> Rect {
        let area = frame.area();
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}
