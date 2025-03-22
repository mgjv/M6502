use crate::computer::cpu;
use crate::App;

use ratatui::buffer::Buffer;
use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

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
            Constraint::Length(19), // left bar
            Constraint::Min(1), // central area
        ])
        .split(frame_chunks[1]);

    let left = Block::bordered().title(" Cpu ");
    let left_area = left.inner(centre_chunks[0]);

    frame.render_widget(left, centre_chunks[0]);

    let left_chunks = Layout::vertical([
            Constraint::Length(4), // status register
            Constraint::Length(4), // A, X and Y
            Constraint::Length(4), // Stack Pointer, Program Counter
            Constraint::Min(1),
        ])
        .split(left_area);

    let status = StatusRegister::new(app.get_cpu_state().status).centered();
    frame.render_widget(status, left_chunks[0]);

    let register_chunks = Layout::horizontal([
            Constraint::Min(1),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(1),
        ])
        .flex(Flex::SpaceAround)
        .split(left_chunks[1]);

    frame.render_widget(Register::new("A".to_string(), app.get_cpu_state().accumulator), register_chunks[1]);
    frame.render_widget(Register::new("X".to_string(), app.get_cpu_state().x_index), register_chunks[2]);
    frame.render_widget(Register::new("Y".to_string(), app.get_cpu_state().y_index), register_chunks[3]);

    let sp_and_pc_chunks = Layout::horizontal([
            Constraint::Min(1),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Min(1),
        ])
        .flex(Flex::SpaceAround)
        .split(left_chunks[2]);

    frame.render_widget(Register::new("SP".to_string(), app.get_cpu_state().stack_pointer), sp_and_pc_chunks[1]);
    frame.render_widget(Address::new("PC".to_string(), app.get_cpu_state().program_counter), sp_and_pc_chunks[2]);

    let right = Block::new().title(" Memory ");

    frame.render_widget(right, centre_chunks[1]);
}

// TODO Address and Register are very similar
struct Address {
    name: String,
    address: u16,
}

impl Address {
    fn new(name: String, address: u16) -> Self {
        Self { name, address }
    }
}

impl Widget for Address {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();
        buf.set_string(area.x, area.y, &self.name, style);
        buf.set_string(area.x, area.y + 1, format!("{:04x}", self.address), style);
    }
}

struct Register {
    name: String,
    value: u8,
}

impl Register {
    fn new(name: String, value: u8) -> Self {
        Self { name, value }
    }
}

impl Widget for Register {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();
        buf.set_string(area.x, area.y, &self.name, style);
        buf.set_string(area.x, area.y + 1, format!("{:02x}", self.value), style);
    }
}

#[derive(Clone, Copy)]
struct StatusRegister {
    status: cpu::status::Status,
    centered_x: bool,
    centered_y: bool,
}

impl StatusRegister {
    fn new(status: cpu::status::Status) -> Self {
        Self {
            status,
            centered_x: false,
            centered_y: false,
        }
    }

    #[allow(dead_code)]
    fn centered(self) -> Self {
        self.centered_x().centered_y()
    }

    fn centered_x(mut self) -> Self {
        self.centered_x = true;
        self
    }

    fn centered_y(mut self) -> Self {
        self.centered_y = true;
        self
    }
}

#[derive(Clone, Copy)]
struct StatusBit {
    value: bool,
    name: char,
}

// his widget displays the value of a status register.
// If the space is narrower than 8, it will simpluy display a
// hex value, under a header SR. If larger than 8, it will display
// 8 bit statuses next to each other.
impl Widget for StatusRegister {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(Color::Yellow);

        // TODO Maybe use internal ratatui Layout functionality

        if area.width < 8 {
            // center if necessary
            let area = Rect::new(
                if self.centered_x { area.x + (area.width / 2) - 1 } else { area.x },
                if self.centered_y { area.y + (area.height / 2) - 1 } else { area.y },
                2,
                area.height,
            );
            buf.set_string(area.x, area.y, "SR", style);
            buf.set_string(area.x, area.y + 1, format!("{:02x}", self.status.as_byte()), style);
        } else {
            let bitwidth = area.width / 8;
            let area = Rect::new(
                if self.centered_x { area.x + (area.width / 2) - (bitwidth * 4) + 1 } else { area.x },
                if self.centered_y { area.y + (area.height / 2) - 1 } else { area.y },
                2,
                area.height,
            );
            let bits = [
                StatusBit { value: self.status.negative, name: 'N' },
                StatusBit { value: self.status.overflow, name: 'V' },
                StatusBit { value: self.status.ignored,  name: '-' },
                StatusBit { value: self.status.brk, name: 'B' },
                StatusBit { value: self.status.decimal, name: 'D' },
                StatusBit { value: self.status.irq_disable, name: 'I' },
                StatusBit { value: self.status.zero, name: 'Z' },
                StatusBit { value: self.status.carry, name: 'C' },
            ];
            for (i, bit) in bits.iter().enumerate() {
                let area = Rect::new(
                    area.x + (i as u16 * bitwidth),
                    area.y,
                    bitwidth,
                    area.height,
                );
                bit.render(area, buf);
            }
        }
    }
}

impl Widget for StatusBit {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(if self.value { Color::LightYellow } else { Color::Gray });
        buf.set_string(area.x, area.y, self.name.to_string(), style);
        buf.set_string(area.x, area.y + 1, (self.value as u8).to_string(), style);
    }
}

