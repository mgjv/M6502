use crate::computer::cpu;

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

#[derive(Clone, Copy)]
pub struct StatusRegisterWidget {
    status: cpu::status::Status,
    centered_x: bool,
    centered_y: bool,
}

impl StatusRegisterWidget {
    pub fn new(status: cpu::status::Status) -> Self {
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

// This widget displays the value of a status register.
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

