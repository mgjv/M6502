use bitflags::bitflags;
use smart_default::SmartDefault;

use super::clock::TickCount;

// The CPU
#[derive(Debug, SmartDefault)]
pub struct CPU {
    // data_bus: u16,
    // address_bus: u16,

    // accumulator: u8,
    // x_index: u8,
    // y_index: u8,

    // stack_pointer: u16,
    program_counter: u16,

    // flags: u8,
    // typically 1 - 3 MHz
}


impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    /* Run for one clock cycle */
    pub fn fetch_and_execute(&mut self) -> Option<TickCount> {
        
        self.program_counter += 1;

        if self.program_counter > 10 {
            return None;
        }
        Some(1)
    }

    // fn fetch(&self) {}

    // fn execute(&self) {}
}

bitflags! {
    pub struct Flags: u8 {
        const PS_NEGATIVE           = 0b1000_0000;
        const PS_OVERFLOW           = 0b0100_0000;
        const PS_UNUSED             = 0b0010_0000; // JAM: Should this exist?
                                                  // (note that it affects the
                                                  // behavior of things like
                                                  // from_bits_truncate)
        const PS_BRK                = 0b0001_0000;
        const PS_DECIMAL_MODE       = 0b0000_1000;
        const PS_DISABLE_INTERRUPTS = 0b0000_0100;
        const PS_ZERO               = 0b0000_0010;
        const PS_CARRY              = 0b0000_0001;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let cpu = CPU::new();
        assert_eq!(cpu.program_counter, 0x0);
    }
}