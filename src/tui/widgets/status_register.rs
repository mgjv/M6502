use crate::computer::cpu;

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

#[derive(Clone, Copy)]
pub struct StatusRegisterWidget {
    status: cpu::status::Status,
}

impl StatusRegisterWidget {
    pub fn new(status: cpu::status::Status) -> Self {
        Self {
            status,
        }
    }
}

// This widget displays the value of a status register.
impl Widget for StatusRegisterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans = vec![];

        spans.push(Span::raw("SR: "));

        spans.push(self.bit_span('N', self.status.negative));
        spans.push(self.bit_span('V', self.status.overflow));
        spans.push(self.bit_span('-', self.status.ignored));
        spans.push(self.bit_span('B', self.status.brk));
        spans.push(self.bit_span('D', self.status.decimal));
        spans.push(self.bit_span('I', self.status.irq_disable));
        spans.push(self.bit_span('Z', self.status.zero));
        spans.push(self.bit_span('C', self.status.carry));

        let line = Line::from(spans);
        line.render(area, buf);
    }
}

impl StatusRegisterWidget {
    fn bit_span(&self, name: char, status: bool) -> Span {
        let style = Style::default().fg(if status { Color::LightYellow } else { Color::DarkGray });
        let name = if status { name.to_string() } else { name.to_lowercase().to_string() };
        Span::styled(name, style)
    }
}
