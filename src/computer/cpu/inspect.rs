use super::*;

// Formatting/Display functions for the CPU type
impl<B: Bus> CPU<B> {

    pub fn show_registers<W: fmt::Write>(&self, b: &mut W) -> Result<(), fmt::Error> {
        write!(b, " A   X   Y")?;
        write!(b, "\tN O {color_bright_black}- B{color_reset} D I Z C")?;
        write!(b, "\t\tNMI  RST  IRQ")?;
        write!(b, "\n")?;

        // compute registers
        write!{b, " {:02X}  {:02X}  {:02X}", 
            self.accumulator, 
            self.x_index, 
            self.y_index}?;

        // status register
        write!(b, "\t{:1b} {:1b} {color_bright_black}{:1b} {:1b}{color_reset} {:1b} {:1b} {:1b} {:1b}", 
            u8::from(self.status.negative),
            u8::from(self.status.overflow),
            u8::from(self.status.ignored),
            u8::from(self.status.brk),
            u8::from(self.status.decimal),
            u8::from(self.status.irq_disable),
            u8::from(self.status.zero),
            u8::from(self.status.carry))?;

        // vectors
        write!(b, "\t\t{:04x} {:04x} {:04x}", 
            self.bus.read_address(NMI_ADDRESS),
            self.bus.read_address(RESET_ADDRESS),
            self.bus.read_address(IRQ_ADDRESS))?;

        write!(b, "\n")?;
        Ok(())
    }

    pub fn show_program_memory<W: fmt::Write>(&self, b: &mut W) -> Result<(), fmt::Error> {
        self.show_memory(b, self.program_counter)
    }

    pub fn show_reset_memory<W: fmt::Write>(&self, b: &mut W) -> Result<(), fmt::Error> {
        let reset_address = self.bus.read_address(RESET_ADDRESS);
        self.show_memory(b, reset_address)
    }

    pub fn show_stack<W: fmt::Write>(&self, b: &mut W) -> Result<(), fmt::Error> {
        let stack_address = bytes_to_address(self.stack_pointer, 0x01);
        self.show_memory(b, stack_address)
    }

    pub fn show_memory<W: fmt::Write>(&self, b: &mut W, focal_address: u16) -> Result<(), fmt::Error> {
        let start = if focal_address > 16 {
            ((focal_address - 16)/ 16) * 16
        } else {
            (focal_address/ 16) * 16
        };
        let end = start + 3 * 16;

        for address in start .. end {
            if address % 16 == 0 {
                write!(b, " {color_bright_blue}0x{:04X}{color_reset}: ", address)?;
            }
            if address % 16 == 8 { write!(b, " ")?; }
            if address == focal_address { write!(b, "{color_red}")?; }

            let byte = self.bus.read_byte(address);
            write!(b, " {:02X}", byte)?;

            if address == focal_address { write!(b, "{color_reset}")?; }

            if address % 16 == 15 { write!(b, "\n")?; }
        }
        Ok(())
    }
}
