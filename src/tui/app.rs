use crate::proxy::ComputerProxy;
use crate::computer::Computer;

pub struct App<'a> {
    pub proxy: ComputerProxy<'a>,
    pub should_quit: bool,

    pub title: String,
    pub version: String,
}

impl<'a> App<'a> {
    pub fn new(computer: &'a Computer) -> Self {
        Self {
            proxy: ComputerProxy::new(computer),
            should_quit: false,

            title: "CMOS 6502 emulator".to_string(),
            version: "0.0.1".to_string(),
        }
    }
}