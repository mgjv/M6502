use ratatui::widgets::Widget;
use ratatui::style::Style;
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;

pub struct RegisterWidget {
    name: String,
    value: u8,
}

impl RegisterWidget {
    pub fn new(name: String, value: u8) -> Self {
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