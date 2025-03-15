use log::{debug, error};
use smart_default::SmartDefault;
use inline_colorization::*;

use std::fmt;

use crate::computer::memory::address_to_bytes;

use super::clock::TickCount;
use super::memory::{Bus, bytes_to_address};

mod instruction;
use instruction::*;

// Standard memory locations to fetch addresses from
const NMI_ADDRESS: u16 = 0xfffa;
const RESET_ADDRESS: u16 = 0xfffc;
const IRQ_ADDRESS: u16 = 0xfffe;

/*
 * For much of the information used here, see
 * https://www.masswerk.at/6502/6502_instruction_set.html
 */

// The CPU
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CPU<B: Bus> {
    pub bus: B,

    accumulator: u8,
    x_index: u8,
    y_index: u8,

    // Stack is on page 1, $0100 - $01ff, runs high -> low
    stack_pointer: u8,
    program_counter: u16,
    status: Status,
}

#[derive(SmartDefault, Debug, PartialEq, Copy, Clone)]
struct Status {
    negative: bool,
    overflow: bool,
    #[default = true] // always appears to be set
    ignored: bool,
    #[default = true] // always appears to be set after reset
    brk: bool, // TODO Hardware interrupts need to set this to false
    decimal: bool,
    irq_disable: bool,
    zero: bool,
    carry: bool
}

impl Status {
    fn as_byte(&self) -> u8 {
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

    fn from_byte(byte: u8) -> Self {
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

impl<B: Bus> CPU<B> {
    pub fn new(bus: B, rom: &[u8]) -> Self {

         let mut new_cpu = Self {
            bus: bus,
            accumulator: 0,
            x_index: 0,
            y_index: 0,

            stack_pointer: 0xfd,

            // program_counter: reset_address,
            program_counter: 0,
            status: Status::default(),
        };

        // Load the rom and execute the reset vector
        new_cpu.load_rom(rom);

        // FIXME this duplicates computer.run() a bit
        // Execute whatever instructions the ROM wants executing
        loop {
            match new_cpu.fetch_and_execute() {
                Some(_) => {},
                None => break,
            }
        }
 

        return new_cpu
    }
}

// Formatting/Display functions
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

impl<B: Bus> CPU<B> {

    // Load the given memory at the end of the address range
    fn load_rom(&mut self, rom: &[u8]) {
        assert!(rom.len() <= 0xffff);
        let start_address: u16 = 0xffff - (rom.len() - 1) as u16;
        debug!("Loading ROM at address {:04x}, length {:04x}", start_address, rom.len());
        self.bus.write_bytes(start_address, rom);
        self.program_counter = self.bus.read_address(RESET_ADDRESS);
        debug!("Setting program counter to {:04x}", self.program_counter);
    }

    pub fn load_program(&mut self, address: u16, program: &[u8]) {
        // TODO put in some better safeguards for a sensible address
        assert!(address > 0x200 && address < 0xfdff);
        //self.bus.write_bytes(0x0000, program);
        debug!("Loading program at address {:04x}, length {:04x}", address, program.len());
        self.bus.write_bytes(address, program);
        self.program_counter = address;
        debug!("Setting program counter to {:04x}", self.program_counter);

    }

    /* Run for one clock cycle */
    pub fn fetch_and_execute(&mut self) -> Option<TickCount> {
        // Read a byte
        let opcode = self.bus.read_byte(self.program_counter);

        // Identify the operator
        // debug!("opcode {:02X} ", opcode);

        // TODO We should get the number of cycles from execute_instruction(), not a static table.

        match decode_instruction(opcode) {
            // FIXME This isn't the right implementation for BRK
            Some((Instruction::BRK, _, _)) => {
                debug!("{:04x}: opcode {:02x} -> BRK/Implied", self.program_counter, opcode);
                self.execute_instruction(Instruction::BRK, Operand::Implied);
                None
            },
            #[cfg(test)]
            Some((Instruction::HALT, _, _)) => {
                debug!("{:04x}: opcode {:02x} -> HALT", self.program_counter, opcode);
                None
            },
            #[cfg(test)]
            Some((Instruction::FAIL, _, _)) => {
                assert!(false, "Address {:04x} contains FAIL.", self.program_counter);
                None
            },
            // Any other valid instruction, we process
            Some((instruction, address_mode, cycles)) => {

                // Fetch given arguments
                let operand_size = address_mode.operand_size();
                let operand_bytes = self.bus.read_two_bytes(self.program_counter + 1);
                let operand = self.get_operand(address_mode, operand_bytes);

                // TODO: provide custom debug stuff for address_mode
                debug!("{:04x}: opcode {:02x} -> {:02x?}/{:02x?}/{:02x?} -> {:02x?} {:02x?}", self.program_counter, opcode, instruction, address_mode, operand_bytes, instruction, operand);

                // Advance the program counter by the correct number of bytes
                // This is done before the instruction is executed, so that the instruction can
                // modify the program counter if needed
                self.program_counter += 1 + operand_size;

                // update the state of memory and CPU
                self.execute_instruction(instruction, operand);

                // FIXME This is here to stop us from running too long. Need to fix
                if self.program_counter > NMI_ADDRESS {
                    error!("Program counter reached {:04X}, halting", self.program_counter);
                    return None;
                }

                // return the number of cycles/ticks consumed
                Some(cycles)
            },
            // Hmm, this is not  valid instruction. Wonder what happened
            None => {
                // This shouldn't happen
                error!("{:04x}: Unused opcode {:02x} found", self.program_counter, opcode);
                // TODO Decide what should happen here in production code
                #[cfg(test)]
                assert!(false);
                None
            },
        }
    }

    pub fn memory_size(&self) -> usize {
        self.bus.memory_size()
    }  

    fn get_operand(&self, addressmode: AddressMode, bytes: [u8; 2]) -> Operand {
        match addressmode {
            AddressMode::Accumulator => Operand::Implied,
            AddressMode::Absolute    => Operand::Address(bytes_to_address(bytes[0], bytes[1])),
            AddressMode::AbsoluteX   => Operand::Address(bytes_to_address(bytes[0], bytes[1]).wrapping_add(self.x_index.into())),
            AddressMode::AbsoluteY   => Operand::Address(bytes_to_address(bytes[0], bytes[1]).wrapping_add(self.y_index.into())),
            AddressMode::Immediate   => Operand::Immediate(bytes[0]),
            AddressMode::Implied     => Operand::Implied,
            AddressMode::Indirect    => {
                let address = bytes_to_address(bytes[0], bytes[1]);
                let bytes = self.bus.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[0], bytes[1]))
            },
            AddressMode::IndirectX   => {
                // Add X to zero page address stored in bytes[0]. Return address stored there
                let address = bytes_to_address(bytes[0].wrapping_add(self.x_index), 0x00);
                let bytes = self.bus.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[0], bytes[1]))
            },
            AddressMode::IndirectY   => {
                // Add contents of Y to address stored in zero page at byte[0] and byte[0] + 1, and return
                let address = bytes_to_address(bytes[0], 0x00);
                let bytes = self.bus.read_two_bytes(address);
                //debug!("read from {:04x}: address bytes: {:02x?}", address, bytes);
                Operand::Address(bytes_to_address(bytes[0], bytes[1]).wrapping_add(self.y_index.into()))
            },
            AddressMode::Relative    => {
                // offset is a 2's complement signed byte
                let offset = bytes[0] as i8;
                // offset is relative to immediate next instruction address         
                let address = self.program_counter.wrapping_add(2).wrapping_add(offset as u16);
                Operand::Address(address)
            },
            AddressMode::Zeropage    => Operand::Address(bytes_to_address(bytes[0], 0)),
            AddressMode::ZeropageX   => Operand::Address(bytes_to_address(bytes[0], 0).wrapping_add(self.x_index.into())),
            AddressMode::ZeropageY   => Operand::Address(bytes_to_address(bytes[0], 0).wrapping_add(self.y_index.into())),
        }
    }

    fn execute_instruction(&mut self, instruction: Instruction, operand: Operand) {

        match instruction {
            Instruction::ADC => {
                match operand {
                    Operand::Immediate(value) => {
                        self.add_with_carry(value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.add_with_carry(value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::AND => {
                match operand {
                    Operand::Immediate(value) => {
                        self.set_accumulator(self.accumulator & value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.set_accumulator(self.accumulator & value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::ASL => {
                match operand {
                    Operand::Implied => {
                        self.status.carry = self.accumulator & 0x80 != 0;
                        self.set_accumulator(self.accumulator << 1);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.status.carry = value & 0x80 != 0;
                        self.store_at_address(address, value << 1);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::BCC => { 
                if !self.status.carry {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::BCS => { 
                if self.status.carry {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::BEQ => {
                if self.status.zero {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::BIT => todo!(),
            Instruction::BMI => {
                if self.status.negative {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::BNE => {
                if !self.status.zero {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::BPL => {
                if !self.status.negative {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::BRK => { self.execute_brk(); },
            Instruction::BVC => {
                if !self.status.overflow {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::BVS => {
                if self.status.overflow {
                    self.do_jump(instruction, operand);
                }
            },
            Instruction::CLC => { self.status.carry = false; },
            Instruction::CLD => { self.status.decimal = false; },
            Instruction::CLI => { self.status.irq_disable = false; },
            Instruction::CLV => { self.status.overflow = false; },
            Instruction::CMP => {
                match operand {
                    Operand::Immediate(value) => {
                        self.compare(self.accumulator, value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.compare(self.accumulator, value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::CPX => {
                match operand {
                    Operand::Immediate(value) => {
                        self.compare(self.x_index, value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.compare(self.x_index, value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::CPY => {
                match operand {
                    Operand::Immediate(value) => {
                        self.compare(self.y_index, value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.compare(self.y_index, value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::DEC => {
                match operand {
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        let new_value = value.wrapping_sub(1);
                        self.update_zero_and_negative_flags(new_value);
                        self.bus.write_byte(address, new_value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::DEX => {
                let new_x = self.x_index.wrapping_sub(1);
                self.set_x_index(new_x);
            },
            Instruction::DEY => {
                let new_y = self.y_index.wrapping_sub(1);
                self.set_y_index(new_y); 
            },
            Instruction::EOR => {
                match operand {
                    Operand::Immediate(value) => {
                        self.set_accumulator(self.accumulator ^ value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.set_accumulator(self.accumulator ^ value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::INC => {
                match operand {
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        let new_value = value.wrapping_add(1);
                        self.update_zero_and_negative_flags(new_value);
                        self.bus.write_byte(address, new_value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::INX => { self.set_x_index(self.x_index.wrapping_add(1)) },
            Instruction::INY => { self.set_y_index(self.y_index.wrapping_add(1)) },
            Instruction::JMP => { self.do_jump(instruction, operand); },
            Instruction::JSR => todo!(),
            Instruction::LDA => {
                match operand {
                    Operand::Immediate(value) => {
                        self.set_accumulator(value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.set_accumulator(value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::LDX => {
                match operand {
                    Operand::Immediate(value) => {
                        self.set_x_index(value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.set_x_index(value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::LDY => {
                match operand {
                    Operand::Immediate(value) => {
                        self.set_y_index(value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.set_y_index(value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::LSR => {
                match operand {
                    Operand::Implied => {
                        self.status.carry = self.accumulator & 0x01 != 0;
                        self.set_accumulator(self.accumulator >> 1);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.status.carry = value & 0x01 != 0;
                        self.store_at_address(address, value >> 1);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::NOP => { /* doesn't do anything */ },
            Instruction::ORA => {
                match operand {
                    Operand::Immediate(value) => {
                        self.set_accumulator(self.accumulator | value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.set_accumulator(self.accumulator | value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::PHA => { self.push_stack(self.accumulator); },
            Instruction::PHP => { 
                let mut status = self.status;
                status.brk = true;
                // debug!("PHP: pushing status %{:08b}", status.as_byte());
                self.push_stack(status.as_byte());
            },
            Instruction::PLA => { 
                let value = self.pull_stack();
                self.set_accumulator(value); 
            },
            Instruction::PLP => { 
                let value = self.pull_stack();
                self.status = Status::from_byte(value);
                // BRK flag should be cleared on pull
                self.status.brk = false;
                // debug!("PLP: setting status %{:08b}", self.status.as_byte());
            },
            Instruction::ROL => {
                match operand {
                    Operand::Implied => {
                        let carry = self.status.carry;
                        self.status.carry = self.accumulator & 0x80 != 0;
                        self.set_accumulator((self.accumulator << 1) | (if carry { 1 } else { 0 }));
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        let carry = self.status.carry;
                        self.status.carry = value & 0x80 != 0;
                        self.store_at_address(address, (value << 1) | (if carry { 1 } else { 0 }));
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::ROR => {
                match operand {
                    Operand::Implied => {
                        let carry = self.status.carry;
                        self.status.carry = self.accumulator & 0x01 != 0;
                        self.set_accumulator((self.accumulator >> 1) | (if carry { 0x80 } else { 0 }));
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        let carry = self.status.carry;
                        self.status.carry = value & 0x01 != 0;
                        self.store_at_address(address, (value >> 1) | (if carry { 0x80 } else { 0 }));
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::RTI => { self.return_from_interrupt(); },
            Instruction::RTS => todo!(),
            Instruction::SBC => {
                match operand {
                    Operand::Immediate(value) => {
                        self.subtract_with_carry(value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.subtract_with_carry(value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::SEC => { self.status.carry = true; },
            Instruction::SED => { self.status.decimal = true; },
            Instruction::SEI => { self.status.irq_disable = true; },
            Instruction::STA => {
                match operand {
                    Operand::Address(address) => {
                        self.bus.write_byte(address, self.accumulator);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::STX => {
                match operand {
                    Operand::Address(address) => {
                        self.bus.write_byte(address, self.x_index);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::STY => {
                match operand {
                    Operand::Address(address) => {
                        self.bus.write_byte(address, self.y_index);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::TAX => { self.x_index = self.accumulator; },
            Instruction::TAY => { self.y_index = self.accumulator; },
            Instruction::TSX => { self.x_index = self.stack_pointer; },
            Instruction::TXA => { self.accumulator = self.x_index; },
            Instruction::TXS => { self.stack_pointer = self.x_index; },
            Instruction::TYA => { self.accumulator = self.y_index; },
            #[cfg(test)]
            Instruction::VRFY => {
                match operand {
                    Operand::Address(address) => {
                        self.verify_test(address);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            #[cfg(test)]
            Instruction::FAIL | Instruction::HALT => { 
                assert!(false, "{} should already have been handled before this", instruction); 
            },
        }
    }

    fn execute_brk(&mut self) {
        let bytes = address_to_bytes(self.program_counter.wrapping_add(2));
        self.push_stack(bytes[1]);
        self.push_stack(bytes[0]);
        self.push_stack(self.status.as_byte());
        self.status.brk = true;
        self.program_counter = self.bus.read_address(NMI_ADDRESS);
    }

    #[allow(dead_code)]
    pub fn execute_nmi(&mut self) {
        self.prepare_for_hardware_interrupt();
        self.program_counter = self.bus.read_address(NMI_ADDRESS);
    }

    #[allow(dead_code)]
    pub fn execute_irq(&mut self) {
        if self.status.irq_disable {
            return;
        }
        self.prepare_for_hardware_interrupt();
        self.program_counter = self.bus.read_address(IRQ_ADDRESS);
    }

    #[allow(dead_code)]
    fn prepare_for_hardware_interrupt(&mut self) {
        let address_bytes = address_to_bytes(self.program_counter);
        self.push_stack(address_bytes[1]); // high byte
        self.push_stack(address_bytes[0]); // low byte
        self.push_stack(self.status.as_byte());
        self.status.brk = false;
    }

    fn return_from_interrupt(&mut self) {
        let status = self.pull_stack();
        let low = self.pull_stack();
        let high = self.pull_stack();
        self.program_counter = bytes_to_address(high, low);
        self.status = Status::from_byte(status);
        // Ensure brk is always false after a hardware restore
        self.status.brk = false;
    }

    // Push a new value on the stack, decrementing the stack pointer
    fn push_stack(&mut self, value: u8) {
        let address = bytes_to_address(self.stack_pointer, 0x01);
        self.bus.write_byte(address, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    // Pull the 'top' value off the stack, incrementing the stack pointer
    fn pull_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let address = bytes_to_address(self.stack_pointer, 0x01);
        self.bus.read_byte(address)
    }

    #[cfg(test)]
    // get the value on the stack at position n, where n = 1 is the 'top'
    fn peek_stack(&self, position: u8) -> u8 {
        let stack_position = self.stack_pointer.wrapping_add(position);
        let address = bytes_to_address(stack_position, 0x01);
        self.bus.read_byte(address)
    }

    fn do_jump(&mut self, instruction: Instruction, operand: Operand) {
        match operand {
            Operand::Address(address) => {
                // debug!("Jumping to {:04x}", address);
                self.program_counter = address;
            },
            _ => illegal_opcode(instruction, operand),
        }
    }

    /* Functions to update registers and addresses, maintaining status flags */

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        self.status.zero = value == 0;
        self.status.negative = value & 0x80 != 0;
    }

    fn set_accumulator(&mut self, value: u8) {
        self.accumulator = value;
        self.update_zero_and_negative_flags(value);
    }

    fn set_x_index(&mut self, value: u8) {
        self.x_index = value;
        self.update_zero_and_negative_flags(value);
    }

    fn set_y_index(&mut self, value: u8) {
        self.y_index = value;
        self.update_zero_and_negative_flags(value);
    }

    fn store_at_address(&mut self, address: u16, value: u8) {
        self.bus.write_byte(address, value);
        self.update_zero_and_negative_flags(value);
    }

    /* More complex operations than fit in a few lines */

    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);
        self.status.carry = register >= value;
        // debug!("Comparing {:02x} and {:02x} resulting in {:02x}", register, value, result);
        self.update_zero_and_negative_flags(result);
    }

    // FIXME Implement decimal mode
    fn add_with_carry(&mut self, value: u8) {
        let a = self.accumulator;
        let c = u8::from(self.status.carry); // either 0 or 1

        let new_a = a.wrapping_add(value).wrapping_add(c);

        self.status.carry = new_a < a 
                    || (new_a == 0 && c == 1) 
                    || (value == 0xff && c == 1);
        self.status.overflow = (a > 0x7f && value > 0x7f && new_a < 0x80) 
                    || (a < 0x80 && value < 0x80 && new_a > 0x7f);

        self.set_accumulator(new_a);
    }

    // FIXME Implement decimal mode
    // FIXME This panics if value == 0xff and carry is not set
    fn subtract_with_carry(&mut self, value: u8) {
        let a = self.accumulator;
        let c = u8::from(self.status.carry); // either 0 or 1

        let new_a = a.wrapping_add(!value).wrapping_add(c);

        self.status.carry = a >= value;
        // self.status.carry = new_a > a
        //             || (new_a < a && c == 1)
        //             || (value == 0xff && c == 0);
        self.status.overflow = (a ^ new_a) & 0x80 != 0 && (a ^ value) & 0x80 != 0;

        self.set_accumulator(new_a);
    }
}

fn illegal_opcode(instruction: Instruction, operand: Operand) {
    error!("Attempt to execute illegal opcode for {:?} with operand {:?}", instruction, operand);
}

#[cfg(test)]
pub mod tests {
    use crate::computer::memory::Memory;
    use std::fmt::Write;
    use super::*;
    use log::debug;

    pub const TEST_ROM: &'static[u8] = 
        &[ 0xa2, 0xff, 0x9a, 0x00, 0x00, 0x00, 0x00, 0x00,
           0x00, 0x00, 0x00, 0x00, 0xf0, 0xff, 0x00, 0x00 ];
    
    fn test_rom_start() -> u16 {
        0xffff - (TEST_ROM.len() - 1) as u16
    }

    fn test_rom_end_of_execution () -> u16 {
        // find the first 0x00
        let offset = TEST_ROM.iter().position(|&b| b == 0x00).unwrap() as u16;
        test_rom_start() + offset as u16
    }

    #[derive(Debug, PartialEq)]
    enum TestOp {
        TestStart,
        TestEnd,

        TestA,
        TestX,
        TestY,
        TestSP,

        TestCarrySet,
        TestCarryClear,
        TestZeroSet,
        TestZeroClear,
        TestNegativeSet,
        TestNegativeClear,
        TestOverflowSet,
        TestOverflowClear,
        TestDecimalSet,
        TestDecimalClear,
        TestInterruptSet,
        TestInterruptClear,
        TestBreakSet,
        TestBreakClear,

        TestAddressContents,
        TestStackContents,
        TestStackPointer,
    }

    impl TryFrom<u8> for TestOp {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0xc2 => Ok(TestOp::TestStart),
                0x00 => Ok(TestOp::TestEnd),
                0x01 => Ok(TestOp::TestA),
                0x02 => Ok(TestOp::TestX),
                0x03 => Ok(TestOp::TestY),
                0x08 => Ok(TestOp::TestSP),
                0x30 => Ok(TestOp::TestCarrySet),
                0x31 => Ok(TestOp::TestCarryClear),
                0x32 => Ok(TestOp::TestZeroSet),
                0x33 => Ok(TestOp::TestZeroClear),
                0x34 => Ok(TestOp::TestNegativeSet),
                0x35 => Ok(TestOp::TestNegativeClear),
                0x36 => Ok(TestOp::TestOverflowSet),
                0x37 => Ok(TestOp::TestOverflowClear),
                0x38 => Ok(TestOp::TestDecimalSet),
                0x39 => Ok(TestOp::TestDecimalClear),
                0x3a => Ok(TestOp::TestInterruptSet),
                0x3b => Ok(TestOp::TestInterruptClear),
                0x3c => Ok(TestOp::TestBreakSet),
                0x3d => Ok(TestOp::TestBreakClear),
                0x80 => Ok(TestOp::TestAddressContents),
                0x88 => Ok(TestOp::TestStackContents),
                0x89 => Ok(TestOp::TestStackPointer),
                _ => Err(()),
            }
        }
    }

    // Add some methods to be used in integration tests in computer
    impl <B: Bus> CPU<B> {
        // This is called by the pseudo test instruction VRFY
        // The test parameters start at the given address
        pub fn verify_test(&self, start_address: u16) {

            let first_op_code = self.bus.read_byte(start_address);
            let first_op = TestOp::try_from(first_op_code).expect(
                &format!("Invalid test operation {:02x} at address {:04x}", first_op_code, start_address)
            );
            assert!(first_op == TestOp::TestStart, 
                "Invalid test start byte {:02x} at address {:04x}", self.bus.read_byte(start_address), start_address);
            let test_id = self.bus.read_byte(start_address + 1);

            debug!("{:04x}: Verifying test with id {:02x}", start_address, test_id);

            let mut address = start_address + 2;
            let mut op_num = 1;
            loop {
                let test_op_code = self.bus.read_byte(address);
                let test_op = TestOp::try_from(test_op_code).expect(
                    &format!("Invalid test operation {:02x} at address {:04x}", test_op_code, address)
                );
                debug!("Test operation: {:?}", test_op);
                match test_op {
                    TestOp::TestStart => assert!(false, "Nested test start at address {:04x}", address),
                    TestOp::TestEnd => break,
                    TestOp::TestA => {
                        address += 1;
                        // assert_eq!(self.accumulator, self.bus.read_byte(address));
                        assert!(self.accumulator == self.bus.read_byte(address),
                            "({:02x}:{:02x}) Assertion failed on Accumulator: \nAccumulator:\t{:02x}\nExpected:\t{:02x}\n\n", 
                            test_id, op_num, self.accumulator, self.bus.read_byte(address));
                    },
                    TestOp::TestX  => { 
                        address += 1;
                        assert!(self.x_index == self.bus.read_byte(address), 
                            "({:02x}:{:02x}) Assertion failed on X Index: \nX Val:\t{:02x}\nExp:\t{:02x}\n\n", 
                            test_id, op_num,  self.x_index, self.bus.read_byte(address));   
                    },
                    TestOp::TestY  => { 
                        address += 1;
                        assert!(self.y_index == self.bus.read_byte(address), 
                            "({:02x}:{:02x}) Assertion failed on Y Index: \nVal:\t{:02x}\nExp:\t{:02x}\n\n", 
                            test_id, op_num,  self.y_index, self.bus.read_byte(address));
                    },
                    TestOp::TestSP  => { 
                        address += 1;
                        assert!(self.stack_pointer == self.bus.read_byte(address),     
                            "({:02x}:{:02x}) Assertion failed on Stack Pointer: \nVal:\t{:02x}\nExp:\t{:02x}\n\n", 
                            test_id, op_num,  self.stack_pointer, self.bus.read_byte(address));
                    },
                    TestOp::TestCarrySet => assert!(self.status.carry, "Carry flag should be set"),
                    TestOp::TestCarryClear => assert!(!self.status.carry, "Carry flag should not be set"),
                    TestOp::TestZeroSet => assert!(self.status.zero, "Zero flag should be set"),
                    TestOp::TestZeroClear => assert!(!self.status.zero, "Zero flag should not be set"),
                    TestOp::TestNegativeSet => assert!(self.status.negative, "Negative flag should be set"),
                    TestOp::TestNegativeClear => assert!(!self.status.negative, "Negative flag should not be set"),
                    TestOp::TestOverflowSet => assert!(self.status.overflow, "Overflow flag should be set"),
                    TestOp::TestOverflowClear => assert!(!self.status.overflow, "Overflow flag should not be set"),
                    TestOp::TestDecimalSet => assert!(self.status.decimal, "Decimal flag should be set"),
                    TestOp::TestDecimalClear => assert!(!self.status.decimal, "Decimal flag should not be set"),
                    TestOp::TestInterruptSet => assert!(self.status.irq_disable, "Interrupt disable flag should be set"),
                    TestOp::TestInterruptClear => assert!(!self.status.irq_disable, "Interrupt disable flag should not be set"),
                    TestOp::TestBreakSet => assert!(self.status.brk, "Break flag should be set"),
                    TestOp::TestBreakClear => assert!(!self.status.brk, "Break flag should not be set"),
                    TestOp::TestAddressContents => {
                        address += 1;
                        let test_address = self.bus.read_address(address);
                        address += 2;
                        let expected = self.bus.read_byte(address);
                        let actual = self.bus.read_byte(test_address);

                        assert!(actual == expected,
                            "({:02x}:{:02x}) Assertion failed on memory at address {:04x}: \nVal:\t{:02x}\nExp:\t{:02x}\n\n",
                            test_id, op_num,  test_address, self.bus.read_byte(test_address), expected);
                    },
                    TestOp::TestStackContents => {
                        address += 1;
                        let position = self.bus.read_byte(address);
                        address += 1;
                        let expected = self.bus.read_byte(address);
                        let actual = self.peek_stack(position);

                        assert!(actual == expected,
                            "({:02x}:{:02x}) Assertion failed on stack at position {:02x}: \nVal:\t{:02x}\nExp:\t{:02x}\n\n",
                            test_id, op_num, position, actual, expected);
                    },
                    TestOp::TestStackPointer => {
                        address += 1;
                        let expected = self.bus.read_byte(address);

                        assert!(self.stack_pointer == expected,
                            "({:02x}:{:02x}) Assertion failed on stack pointer: \nVal:\t{:02x}\nExp:\t{:02x}\n\n",
                            test_id, op_num, self.stack_pointer, expected);
                    }
                }
                address += 1;
                op_num += 1;
            }
        }

        #[allow(unused_must_use)]
        #[allow(dead_code)]
        pub fn debug_show(&self) -> String {
            let mut b = String::new();
            writeln!(b, "Registers:\tStatus:");
            self.show_registers(&mut b);
            // writeln!(b, "Program memory (PC location in red):");
            // self.show_program_memory(&mut b);
            // writeln!(b, "Reset memory:");
            // self.show_reset_memory(&mut b);
            writeln!(b, "Stack:");
            self.show_stack(&mut b);
    
            b
        }
    }

    #[test]
    fn verify_test_rom() {
        let cpu = CPU::new(Memory::new(), TEST_ROM);
        let start_address: u16 = test_rom_start();
        let reset_vector = cpu.bus.read_address(RESET_ADDRESS);
        assert_eq!(reset_vector, start_address);

        // XXX Everything after this may need to change when the TEST_ROM changes

        // ensure we start with LDX $ff, TXS
        assert_eq!(cpu.bus.read_byte(start_address), 0xa2);
        assert_eq!(cpu.bus.read_byte(start_address + 1), 0xff);
        assert_eq!(cpu.bus.read_byte(start_address + 2), 0x9a);

        // ensure we know when it stops
        let end = test_rom_end_of_execution();
        assert_eq!(end, start_address + 3);
        assert_eq!(cpu.bus.read_byte(end), 0x00);
    }

    #[test]
    fn creation() {
        let cpu = CPU::new(Memory::new(), TEST_ROM);

        assert_eq!(cpu.bus.read_address(RESET_ADDRESS), test_rom_start());

        // FIXME This expectation is incorrect, since BRK changes PC
        // FIXME Fix this when VerifyTest is properly implemented
        // assert_eq!(cpu.program_counter, test_rom_end_of_execution());
        // assert_eq!(cpu.stack_pointer, 0xff);
        assert_eq!(cpu.accumulator, 0x0);
        // set by TEST_ROM
        assert_eq!(cpu.x_index, 0xff);
        assert_eq!(cpu.y_index, 0x0);
    }

    #[test]
    fn load_program() {
        let program = [0xa9, 0x01, 0x69, 0x02, 0x8d, 0x02];
        let mut cpu = CPU::new(Memory::new(), TEST_ROM);

        // pretend we loaded a ROM with this vector
        let load_address = 0xc000;
        // set_reset_address(&mut cpu.bus, load_address);

        cpu.load_program(load_address, &program);

        for i in 0..program.len() {
            let data = cpu.bus.read_byte(load_address + i as u16);
            assert_eq!(program[i], data);
        }
    }

    #[test]
    fn load_rom() {
        let rom = [0x10, 0x20, 0x30, 0x40];
        let mut cpu = CPU::new(Memory::new(), TEST_ROM);

        cpu.load_rom(&rom);

        assert_eq!(cpu.bus.read_byte(0xffff), 0x40);
        assert_eq!(cpu.bus.read_byte(0xfffe), 0x30);
        assert_eq!(cpu.bus.read_byte(0xfffd), 0x20);
        assert_eq!(cpu.bus.read_byte(0xfffc), 0x10);
    }
}
