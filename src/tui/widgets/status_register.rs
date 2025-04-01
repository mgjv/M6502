use crate::computer::cpu;

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};
use ratatui::layout::Rect;
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

#[derive(Clone, Copy)]
struct StatusBitWidget {
    value: bool,
    name: char,
}

// This widget displays the value of a status register.
// If the space is narrower than 8, it will simpluy display a
// hex value, under a header SR. If larger than 8, it will display
// 8 bit statuses next to each other.
impl Widget for StatusRegisterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
                area.x + (i as u16),
                area.y,
                1,
                area.height,
            );
            bit.render(area, buf);
        }

    }
}

impl Widget for StatusBitWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(if self.value { Color::LightYellow } else { Color::DarkGray });
        let name = if self.value { self.name.to_string() } else { self.name.to_lowercase().to_string() };
        buf.set_string(area.x, area.y, name, style);
    }
}
