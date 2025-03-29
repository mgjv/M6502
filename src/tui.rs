use crate::computer::cpu;
use crate::App;

use ratatui::buffer::Buffer;
use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Widget};

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

struct MemoryWidget<'a> {
    app: &'a App<'a>,
    start: u16,
    focus: u16,
}

impl<'a> MemoryWidget<'a> {
    fn new(app: &'a App, start: u16) -> Self {
        // Adjust start to nearest lower boundary
        Self {
            app,
            start,
            focus: 0x0000,
        }
    }

    fn set_focus(mut self, focus: u16) -> Self {
        self.focus = focus;
        self
    }
}

impl Widget for MemoryWidget<'_> {
    // TODO Check area boundaries
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.app.get_memory_lines(self.start);

        for (i, (address, line)) in lines.iter().enumerate() {
            let line_area = Rect::new(area.x, area.y + i as u16, area.width, 1);
            let style = Style::default();

            let mut spans = vec![];
            spans.push(Span::from(format!("{address:04x}")));
            spans.push(Span::from(": " ));

            for (j, value) in line.iter().enumerate() {
                // TODO We probably should use cpu::instruction::decode_instruction()
                // and cpu::instruction::AddressMode::get_operand_size()
                // to determine how many bytes to colour, assuming this is an instruction
                let style = if address + j as u16 == self.focus {
                    style.bg(Color::Red)
                } else {
                    style
                };
                spans.push(Span::from(format!("{value:02x}")).style(style));
                spans.push(Span::from(" ".to_string()));
            }

            let line = Line::from(spans);
            line.render(line_area, buf);
        }
    }
}

// TODO Address and Register are very similar
struct AddressWidget {
    name: String,
    address: u16,
}

impl AddressWidget {
    fn new(name: String, address: u16) -> Self {
        Self { name, address }
    }
}

impl Widget for AddressWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();
        buf.set_string(area.x, area.y, &self.name, style);
        buf.set_string(area.x, area.y + 1, format!("{:04x}", self.address), style);
    }
}

struct RegisterWidget {
    name: String,
    value: u8,
}

impl RegisterWidget {
    fn new(name: String, value: u8) -> Self {
        Self { name, value }
    }
}

impl Widget for RegisterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();
        buf.set_string(area.x, area.y, &self.name, style);
        buf.set_string(area.x, area.y + 1, format!("{:02x}", self.value), style);
    }
}

#[derive(Clone, Copy)]
struct StatusRegisterWidget {
    status: cpu::status::Status,
    centered_x: bool,
    centered_y: bool,
}

impl StatusRegisterWidget {
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
struct StatusBitWidget {
    value: bool,
    name: char,
}

// his widget displays the value of a status register.
// If the space is narrower than 8, it will simpluy display a
// hex value, under a header SR. If larger than 8, it will display
// 8 bit statuses next to each other.
impl Widget for StatusRegisterWidget {
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
                StatusBitWidget { value: self.status.negative, name: 'N' },
                StatusBitWidget { value: self.status.overflow, name: 'V' },
                StatusBitWidget { value: self.status.ignored,  name: '-' },
                StatusBitWidget { value: self.status.brk, name: 'B' },
                StatusBitWidget { value: self.status.decimal, name: 'D' },
                StatusBitWidget { value: self.status.irq_disable, name: 'I' },
                StatusBitWidget { value: self.status.zero, name: 'Z' },
                StatusBitWidget { value: self.status.carry, name: 'C' },
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

impl Widget for StatusBitWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(if self.value { Color::LightYellow } else { Color::Gray });
        buf.set_string(area.x, area.y, self.name.to_string(), style);
        buf.set_string(area.x, area.y + 1, (self.value as u8).to_string(), style);
    }
}

