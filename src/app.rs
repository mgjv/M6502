use crate::computer::Computer;

pub struct App<'a> {
    // Some display stuff
    pub title: String,
    pub version: String,

    // A reference to the computer we're supposed to shadow
    pub computer: &'a Computer,
}

impl<'a> App<'a> {
    pub fn new(computer: &'a Computer) -> Self {
        Self {
            computer,
            title: "CMOS 6502 emulator".to_string(),
            version: "0.0.1".to_string(),
        }
    }
}