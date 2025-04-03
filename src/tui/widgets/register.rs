use ratatui::text::Line;
use ratatui::widgets::Widget;
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;

// TODO Address and Register are very similar
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
        let line = Line::raw(format!("{}:{:02x}", self.name, self.value));
        line.render(area, buf);
    }
}