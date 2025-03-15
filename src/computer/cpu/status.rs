use smart_default::SmartDefault;

#[derive(SmartDefault, Debug, PartialEq, Copy, Clone)]
pub struct Status {
    pub negative: bool,
    pub overflow: bool,
    #[default = true] // always appears to be set
    pub ignored: bool,
    #[default = true] // always appears to be set after reset
    pub brk: bool, // TODO Hardware interrupts need to set this to false
    pub decimal: bool,
    pub irq_disable: bool,
    pub zero: bool,
    pub carry: bool
}

impl Status {
    pub fn as_byte(&self) -> u8 {
        let mut byte: u8 = 0;
        byte |= u8::from(self.negative) << 7;
        byte |= u8::from(self.overflow) << 6;
        byte |= u8::from(self.ignored) << 5;
        byte |= u8::from(self.brk) << 4;
        byte |= u8::from(self.decimal) << 3;
        byte |= u8::from(self.irq_disable) << 2;
        byte |= u8::from(self.zero) << 1;
        byte |= u8::from(self.carry);
        byte
    }

    pub fn from_byte(byte: u8) -> Self {
        Self {
            negative: (byte & 0b1000_0000) != 0,
            overflow: (byte & 0b0100_0000) != 0,
            ignored: (byte & 0b0010_0000) != 0,
            brk: (byte & 0b0001_0000) != 0,
            decimal: (byte & 0b0000_1000) != 0,
            irq_disable: (byte & 0b0000_0100) != 0,
            zero: (byte & 0b0000_0010) != 0,
            carry: (byte & 0b0000_0001) != 0,
        }
    }
}
