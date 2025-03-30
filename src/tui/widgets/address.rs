use ratatui::widgets::Widget;
use ratatui::style::Style;
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;

// TODO Address and Register are very similar
pub struct AddressWidget {
    name: String,
    address: u16,
}

impl AddressWidget {
    pub fn new(name: String, address: u16) -> Self {
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

