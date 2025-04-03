use ratatui::text::Line;
use ratatui::widgets::Widget;
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
        let line = Line::raw(format!("{}: {:04x}", self.name, self.address));
        line.render(area, buf);
    }
}

