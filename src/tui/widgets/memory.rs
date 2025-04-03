use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use ratatui::style::{Color, Style};
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;

use crate::app::App;

pub struct MemoryWidget<'a> {
    app: &'a App<'a>,
    start: u16,
    focus: u16,
}

impl<'a> MemoryWidget<'a> {
    pub fn new(app: &'a App, start: u16) -> Self {
        // Adjust start to nearest lower boundary
        Self {
            app,
            start,
            focus: 0x0000,
        }
    }

    pub fn set_focus(mut self, focus: u16) -> Self {
        self.focus = focus;
        self
    }
}

impl Widget for MemoryWidget<'_> {
    // TODO Check area boundaries
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.app.get_memory_lines(self.start, 12, 16);

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
