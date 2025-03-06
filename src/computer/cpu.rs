use log::{debug, error};
use smart_default::SmartDefault;
use inline_colorization::*;

use std::fmt;

use super::clock::TickCount;
use super::memory::{Bus, bytes_to_address};

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
    bus: B,

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
    brk: bool,
    decimal: bool,
    irq_disable: bool,
    zero: bool,
    carry: bool
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

        match decode_instruction(opcode) {
            // If we get a BRK, we halt execution
            Some((Instruction::BRK, _, _)) => {
                None
            }
            // Any other valid instruction, we process
            Some((instruction, address_mode, cycles)) => {

                // Fetch given arguments
                let operand_size = address_mode.operand_size();
                let operand_bytes = self.bus.read_two_bytes(self.program_counter + 1);
                let operand = self.get_operand(address_mode, operand_bytes);

                debug!("opcode {:02x} -> {:x?}/{:x?}/{:x?} -> {:x?} {:x?}", opcode, instruction, address_mode, operand_bytes, instruction, operand);

                // update the state of memory and CPU
                self.execute(instruction, operand);

                // Advance the program counter by the correct number of bytes
                self.program_counter += 1 + operand_size;

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
                error!("Unused opcode {:02X} found at address {:04X}", opcode, self.program_counter);
                None
            }
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
            // FIXME TRIPLE check the next three. Byte order for addresses and logic
            AddressMode::Indirect    => {
                let address = bytes_to_address(bytes[0], bytes[1]);
                let bytes = self.bus.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[0], bytes[1]))
            },
            AddressMode::IndirectX   => {
                // Add X to zero page address stored in bytes[0]. Return address stored there
                // TODO If bytes[0] is 0xfe, this is probably not right, as it will read past the zero page
                let address = bytes_to_address(0, bytes[0].wrapping_add(self.x_index));
                let bytes = self.bus.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[0], bytes[1]))
            },
            AddressMode::IndirectY   => {
                // Add contents of Y to address stored in zero page at byte[0] and byte[0] + 1, and return
                let address = bytes_to_address(0, bytes[0]);
                let bytes = self.bus.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[0], bytes[1]).wrapping_add(self.y_index.into()))
            },
            // FIXME All of the below need to still be implemented
            AddressMode::Relative    => Operand::Implied,
            AddressMode::Zeropage    => Operand::Address(bytes_to_address(bytes[0], 0)),
            AddressMode::ZeropageX   => Operand::Address(bytes_to_address(bytes[0], 0).wrapping_add(self.x_index.into())),
            AddressMode::ZeropageY   => Operand::Address(bytes_to_address(bytes[0], 0).wrapping_add(self.y_index.into())),
        }
    }

    fn execute(&mut self, instruction: Instruction, operand: Operand) {

        match instruction {
            Instruction::ADC => {
                match operand {
                    Operand::Immediate(value) => {
                        self.execute_adc(value);
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.execute_adc(value);
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::AND => todo!(),
            Instruction::ASL => todo!(),
            Instruction::BCC => todo!(),
            Instruction::BCS => todo!(),
            Instruction::BEQ => todo!(),
            Instruction::BIT => todo!(),
            Instruction::BMI => todo!(),
            Instruction::BNE => todo!(),
            Instruction::BPL => todo!(),
            Instruction::BRK => todo!(),
            Instruction::BVC => todo!(),
            Instruction::BVS => todo!(),
            Instruction::CLC => { self.status.carry = false; },
            Instruction::CLD => { self.status.decimal = false; },
            Instruction::CLI => { self.status.irq_disable = false; },
            Instruction::CLV => { self.status.overflow = false; },
            Instruction::CMP => todo!(),
            Instruction::CPX => todo!(),
            Instruction::CPY => todo!(),
            Instruction::DEC => todo!(),
            Instruction::DEX => todo!(),
            Instruction::DEY => todo!(),
            Instruction::EOR => todo!(),
            Instruction::INC => todo!(),
            Instruction::INX => todo!(),
            Instruction::INY => todo!(),
            Instruction::JMP => {
                match operand {
                    Operand::Address(address) => {
                        self.program_counter = address;
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::JSR => todo!(),
            Instruction::LDA => {
                match operand {
                    Operand::Immediate(value) => {
                        self.accumulator = value;
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.accumulator = value;
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::LDX => {
                match operand {
                    Operand::Immediate(value) => {
                        self.x_index = value;
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.x_index = value;
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::LDY => {
                match operand {
                    Operand::Immediate(value) => {
                        self.y_index = value;
                    },
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.y_index = value;
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
            Instruction::LSR => todo!(),
            Instruction::NOP => todo!(),
            Instruction::ORA => todo!(),
            Instruction::PHA => todo!(),
            Instruction::PHP => todo!(),
            Instruction::PLA => todo!(),
            Instruction::PLP => todo!(),
            Instruction::ROL => todo!(),
            Instruction::ROR => todo!(),
            Instruction::RTI => todo!(),
            Instruction::RTS => todo!(),
            Instruction::SBC => todo!(),
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
        }
    }

    fn execute_adc(&mut self, value: u8) {
        let a = self.accumulator;
        let c = u8::from(self.status.carry); // either 0 or 1

        let new_a = a.wrapping_add(value).wrapping_add(c);

        self.status.carry = new_a < a 
                    || (new_a == 0 && c == 0x01) 
                    || (value == 0xff && c == 0x01);
        self.status.overflow = (a > 0x7f && value > 0x7f && new_a < 0x80) 
                    || (a < 0x80 && value < 0x80 && new_a > 0x7f);

        self.accumulator = new_a;
    }

}

fn illegal_opcode(instruction: Instruction, operand: Operand) {
    error!("Attempt to execute illegal opcode for {:?} with operand {:?}", instruction, operand);
}

// Possible address modes for the above instructions
#[derive(Clone, Copy, Debug, PartialEq)]
enum AddressMode {
    Accumulator, // OPC A	    operand is AC (implied single byte instruction)
    Absolute,    // OPC $LLHH	operand is address $HHLL
    AbsoluteX,   // OPC $LLHH,X	operand is address; effective address is address incremented by X with carry **
    AbsoluteY,   // OPC $LLHH,Y	operand is address; effective address is address incremented by Y with carry **
    Immediate,   // OPC #$BB	operand is byte BB
    Implied,     // OPC         operand implied
    Indirect,    // OPC ($LLHH)	operand is address; effective address is contents of word at address: C.w($HHLL)
    IndirectX,   // OPC ($LL,X)	operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
    IndirectY,   // OPC ($LL),Y	operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
    Relative,    // OPC $BB	    branch target is PC + signed offset BB ***
    Zeropage,    // OPC $LL	    operand is zeropage address (hi-byte is zero, address = $00LL)
    ZeropageX,   // OPC $LL,X	operand is zeropage address; effective address is address incremented by X without carry **
    ZeropageY,   // OPC $LL,Y	operand is zeropage address; effective address is address incremented by Y without carry **
}

// What sort of argument unwrapping/fetching may need to happen
#[derive(Clone, Copy, Debug, PartialEq)]
enum Operand {
    Implied,
    Immediate(u8),
    Address(u16),
    Relative(u16),
}

impl AddressMode {
    // Returns the number of bytes needed for the operand
    fn operand_size(&self) -> u16 {
        match self {
            AddressMode::Accumulator => 0,
            AddressMode::Absolute => 2,
            AddressMode::AbsoluteX => 2,
            AddressMode::AbsoluteY => 2,
            AddressMode::Immediate => 1,
            AddressMode::Implied => 0,
            AddressMode::Indirect => 2,
            AddressMode::IndirectX => 1,
            AddressMode::IndirectY => 1,
            AddressMode::Relative => 1,
            AddressMode::Zeropage => 1,
            AddressMode::ZeropageX => 1,
            AddressMode::ZeropageY => 1,
        }
    }
}

// Returns a tuple of the instruction, the addressmode and the (minimum) number of cycles needed
const fn decode_instruction(op_code: u8) -> Option<(Instruction, AddressMode, u8)> {
    match op_code {
        0x00 => Some((Instruction::BRK, AddressMode::Implied, 7)),
        0x01 => Some((Instruction::ORA, AddressMode::IndirectX, 6)),
        0x02 => None,
        0x03 => None,
        0x04 => None,
        0x05 => Some((Instruction::ORA, AddressMode::Zeropage, 3)),
        0x06 => Some((Instruction::ASL, AddressMode::Zeropage, 5)),
        0x07 => None,
        0x08 => Some((Instruction::PHP, AddressMode::Implied, 3)),
        0x09 => Some((Instruction::ORA, AddressMode::Immediate, 2)),
        0x0a => Some((Instruction::ASL, AddressMode::Accumulator, 2)),
        0x0b => None,
        0x0c => None,
        0x0d => Some((Instruction::ORA, AddressMode::Absolute, 4)),
        0x0e => Some((Instruction::ASL, AddressMode::Absolute, 6)),
        0x0f => None,

        0x10 => Some((Instruction::BPL, AddressMode::Relative, 2)),
        0x11 => Some((Instruction::ORA, AddressMode::IndirectY, 5)),
        0x12 => None,
        0x13 => None,
        0x14 => None,
        0x15 => Some((Instruction::ORA, AddressMode::ZeropageX, 4)),
        0x16 => Some((Instruction::ASL, AddressMode::ZeropageX, 6)),
        0x17 => None,
        0x18 => Some((Instruction::CLC, AddressMode::Implied, 2)),
        0x19 => Some((Instruction::ORA, AddressMode::AbsoluteY, 4)),
        0x1a => None,
        0x1b => None,
        0x1c => None,
        0x1d => Some((Instruction::ORA, AddressMode::AbsoluteX, 4)),
        0x1e => Some((Instruction::ASL, AddressMode::AbsoluteX, 7)),
        0x1f => None,

        0x20 => Some((Instruction::JSR, AddressMode::Absolute, 6)),
        0x21 => Some((Instruction::AND, AddressMode::IndirectX, 6)),
        0x22 => None,
        0x23 => None,
        0x24 => Some((Instruction::BIT, AddressMode::Zeropage, 3)),
        0x25 => Some((Instruction::AND, AddressMode::Zeropage, 3)),
        0x26 => Some((Instruction::ROL, AddressMode::Zeropage, 5)),
        0x27 => None,
        0x28 => Some((Instruction::PLP, AddressMode::Implied, 4)),
        0x29 => Some((Instruction::AND, AddressMode::Immediate, 2)),
        0x2a => Some((Instruction::ROL, AddressMode::Accumulator, 2)),
        0x2b => None,
        0x2c => Some((Instruction::BIT, AddressMode::Absolute, 4)),
        0x2d => Some((Instruction::AND, AddressMode::Absolute, 4)),
        0x2e => Some((Instruction::ROL, AddressMode::Absolute, 6)),
        0x2f => None,

        0x30 => Some((Instruction::BMI, AddressMode::Relative, 2)),
        0x31 => Some((Instruction::AND, AddressMode::IndirectY, 5)),
        0x32 => None,
        0x33 => None,
        0x34 => None,
        0x35 => Some((Instruction::AND, AddressMode::ZeropageX, 4)),
        0x36 => Some((Instruction::ROL, AddressMode::ZeropageX, 6)),
        0x37 => None,
        0x38 => Some((Instruction::SEC, AddressMode::Implied, 2)),
        0x39 => Some((Instruction::AND, AddressMode::AbsoluteY, 4)),
        0x3a => None,
        0x3b => None,
        0x3c => None,
        0x3d => Some((Instruction::AND, AddressMode::AbsoluteX, 4)),
        0x3e => Some((Instruction::ROL, AddressMode::AbsoluteX, 7)),
        0x3f => None,

        0x40 => Some((Instruction::RTI, AddressMode::Implied, 6)),
        0x41 => Some((Instruction::EOR, AddressMode::IndirectX, 6)),
        0x42 => None,
        0x43 => None,
        0x44 => None,
        0x45 => Some((Instruction::EOR, AddressMode::Zeropage, 3)),
        0x46 => Some((Instruction::LSR, AddressMode::Zeropage, 5)),
        0x47 => None,
        0x48 => Some((Instruction::PHA, AddressMode::Implied, 3)),
        0x49 => Some((Instruction::EOR, AddressMode::Immediate, 2)),
        0x4a => Some((Instruction::LSR, AddressMode::Accumulator, 2)),
        0x4b => None,
        0x4c => Some((Instruction::JMP, AddressMode::Absolute, 3)),
        0x4d => Some((Instruction::EOR, AddressMode::Absolute, 4)),
        0x4e => Some((Instruction::LSR, AddressMode::Absolute, 6)),
        0x4f => None,

        0x50 => Some((Instruction::BVC, AddressMode::Relative, 2)),
        0x51 => Some((Instruction::EOR, AddressMode::IndirectY, 5)),
        0x52 => None,
        0x53 => None,
        0x54 => None,
        0x55 => Some((Instruction::EOR, AddressMode::ZeropageX, 4)),
        0x56 => Some((Instruction::LSR, AddressMode::ZeropageX, 6)),
        0x57 => None,
        0x58 => Some((Instruction::CLI, AddressMode::Implied, 2)),
        0x59 => Some((Instruction::EOR, AddressMode::AbsoluteY, 4)),
        0x5a => None,
        0x5b => None,
        0x5c => None,
        0x5d => Some((Instruction::EOR, AddressMode::AbsoluteX, 4)),
        0x5e => Some((Instruction::LSR, AddressMode::AbsoluteX, 7)),
        0x5f => None,

        0x60 => Some((Instruction::RTS, AddressMode::Implied, 6)),
        0x61 => Some((Instruction::ADC, AddressMode::IndirectX, 6)),
        0x62 => None,
        0x63 => None,
        0x64 => None,
        0x65 => Some((Instruction::ADC, AddressMode::Zeropage, 3)),
        0x66 => Some((Instruction::ROR, AddressMode::Zeropage, 5)),
        0x67 => None,
        0x68 => Some((Instruction::PLA, AddressMode::Implied, 4)),
        0x69 => Some((Instruction::ADC, AddressMode::Immediate, 2)),
        0x6a => Some((Instruction::ROR, AddressMode::Accumulator, 2)),
        0x6b => None,
        0x6c => Some((Instruction::JMP, AddressMode::Indirect, 5)),
        0x6d => Some((Instruction::ADC, AddressMode::Absolute, 4)),
        0x6e => Some((Instruction::ROR, AddressMode::Absolute, 6)),
        0x6f => None,

        0x70 => Some((Instruction::BVS, AddressMode::Relative, 2)),
        0x71 => Some((Instruction::ADC, AddressMode::IndirectY, 5)),
        0x72 => None,
        0x73 => None,
        0x74 => None,
        0x75 => Some((Instruction::ADC, AddressMode::ZeropageX, 4)),
        0x76 => Some((Instruction::ROR, AddressMode::ZeropageX, 6)),
        0x77 => None,
        0x78 => Some((Instruction::SEI, AddressMode::Implied, 2)),
        0x79 => Some((Instruction::ADC, AddressMode::AbsoluteY, 4)),
        0x7a => None,
        0x7b => None,
        0x7c => None,
        0x7d => Some((Instruction::ADC, AddressMode::AbsoluteX, 4)),
        0x7e => Some((Instruction::ROR, AddressMode::AbsoluteX, 7)),
        0x7f => None,

        0x80 => None,
        0x81 => Some((Instruction::STA, AddressMode::IndirectX, 6)),
        0x82 => None,
        0x83 => None,
        0x84 => Some((Instruction::STY, AddressMode::Zeropage, 3)),
        0x85 => Some((Instruction::STA, AddressMode::Zeropage, 3)),
        0x86 => Some((Instruction::STX, AddressMode::Zeropage, 3)),
        0x87 => None,
        0x88 => Some((Instruction::DEY, AddressMode::Implied, 2)),
        0x89 => None,
        0x8a => Some((Instruction::TXA, AddressMode::Implied, 2)),
        0x8b => None,
        0x8c => Some((Instruction::STY, AddressMode::Absolute, 4)),
        0x8d => Some((Instruction::STA, AddressMode::Absolute, 4)),
        0x8e => Some((Instruction::STX, AddressMode::Absolute, 4)),
        0x8f => None,

        0x90 => Some((Instruction::BCC, AddressMode::Relative, 2)),
        0x91 => Some((Instruction::STA, AddressMode::IndirectX, 6)),
        0x92 => None,
        0x93 => None,
        0x94 => Some((Instruction::STY, AddressMode::ZeropageX, 4)),
        0x95 => Some((Instruction::STA, AddressMode::ZeropageX, 4)),
        0x96 => Some((Instruction::STX, AddressMode::ZeropageY, 4)),
        0x97 => None,
        0x98 => Some((Instruction::TYA, AddressMode::Implied, 2)),
        0x99 => Some((Instruction::STA, AddressMode::AbsoluteY, 5)),
        0x9a => Some((Instruction::TXS, AddressMode::Implied, 2)),
        0x9b => None,
        0x9c => None,
        0x9d => Some((Instruction::STA, AddressMode::AbsoluteX, 5)),
        0x9e => None,
        0x9f => None,

        0xa0 => Some((Instruction::LDY, AddressMode::Immediate, 2)),
        0xa1 => Some((Instruction::LDA, AddressMode::IndirectX, 6)),
        0xa2 => Some((Instruction::LDX, AddressMode::Immediate, 2)),
        0xa3 => None,
        0xa4 => Some((Instruction::LDY, AddressMode::Zeropage, 3)),
        0xa5 => Some((Instruction::LDA, AddressMode::Zeropage, 3)),
        0xa6 => Some((Instruction::LDX, AddressMode::Zeropage, 3)),
        0xa7 => None,
        0xa8 => Some((Instruction::TAY, AddressMode::Implied, 2)),
        0xa9 => Some((Instruction::LDA, AddressMode::Immediate, 2)),
        0xaa => Some((Instruction::TAX, AddressMode::Implied, 2)),
        0xab => None,
        0xac => Some((Instruction::LDY, AddressMode::Absolute, 4)),
        0xad => Some((Instruction::LDA, AddressMode::Absolute, 4)),
        0xae => Some((Instruction::LDX, AddressMode::Absolute, 4)),
        0xaf => None,

        0xb0 => Some((Instruction::BCS, AddressMode::Relative, 2)),
        0xb1 => Some((Instruction::LDA, AddressMode::IndirectY, 5)),
        0xb2 => None,
        0xb3 => None,
        0xb4 => Some((Instruction::LDY, AddressMode::ZeropageX, 4)),
        0xb5 => Some((Instruction::LDA, AddressMode::ZeropageX, 4)),
        0xb6 => Some((Instruction::LDX, AddressMode::ZeropageY, 4)),
        0xb7 => None,
        0xb8 => Some((Instruction::CLV, AddressMode::Implied, 2)),
        0xb9 => Some((Instruction::LDA, AddressMode::Immediate, 2)),
        0xba => Some((Instruction::TSX, AddressMode::Implied, 2)),
        0xbb => None,
        0xbc => Some((Instruction::LDY, AddressMode::AbsoluteX, 4)),
        0xbd => Some((Instruction::LDA, AddressMode::AbsoluteX, 4)),
        0xbe => Some((Instruction::LDX, AddressMode::AbsoluteY, 4)),
        0xbf => None,

        0xc0 => Some((Instruction::CPY, AddressMode::Immediate, 2)),
        0xc1 => Some((Instruction::CMP, AddressMode::IndirectX, 6)),
        0xc2 => None,
        0xc3 => None,
        0xc4 => Some((Instruction::CPY, AddressMode::Zeropage, 3)),
        0xc5 => Some((Instruction::CMP, AddressMode::Zeropage, 3)),
        0xc6 => Some((Instruction::DEC, AddressMode::Zeropage, 5)),
        0xc7 => None,
        0xc8 => Some((Instruction::INY, AddressMode::Implied, 2)),
        0xc9 => Some((Instruction::CMP, AddressMode::Immediate, 2)),
        0xca => Some((Instruction::DEX, AddressMode::Implied, 2)),
        0xcb => None,
        0xcc => Some((Instruction::CPY, AddressMode::Absolute, 4)),
        0xcd => Some((Instruction::CMP, AddressMode::Absolute, 4)),
        0xce => Some((Instruction::DEC, AddressMode::Absolute, 6)),
        0xcf => None,

        0xd0 => Some((Instruction::BNE, AddressMode::Relative, 2)),
        0xd1 => Some((Instruction::CMP, AddressMode::IndirectY, 5)),
        0xd2 => None,
        0xd3 => None,
        0xd4 => None,
        0xd5 => Some((Instruction::CMP, AddressMode::ZeropageX, 4)),
        0xd6 => Some((Instruction::DEC, AddressMode::ZeropageX, 6)),
        0xd7 => None,
        0xd8 => Some((Instruction::CLD, AddressMode::Implied, 2)),
        0xd9 => Some((Instruction::CMP, AddressMode::AbsoluteY, 4)),
        0xda => None,
        0xdb => None,
        0xdc => None,
        0xdd => Some((Instruction::CMP, AddressMode::AbsoluteX, 4)),
        0xde => Some((Instruction::DEC, AddressMode::AbsoluteX, 7)),
        0xdf => None,

        0xe0 => Some((Instruction::CPX, AddressMode::Immediate, 2)),
        0xe1 => Some((Instruction::SBC, AddressMode::IndirectX, 6)),
        0xe2 => None,
        0xe3 => None,
        0xe4 => Some((Instruction::CPX, AddressMode::Zeropage, 3)),
        0xe5 => Some((Instruction::SBC, AddressMode::Zeropage, 3)),
        0xe6 => Some((Instruction::INC, AddressMode::Zeropage, 5)),
        0xe7 => None,
        0xe8 => Some((Instruction::INX, AddressMode::Implied, 2)),
        0xe9 => Some((Instruction::SBC, AddressMode::Immediate, 2)),
        0xea => Some((Instruction::NOP, AddressMode::Implied, 2)),
        0xeb => None,
        0xec => Some((Instruction::CPX, AddressMode::Absolute, 4)),
        0xed => Some((Instruction::SBC, AddressMode::Absolute, 4)),
        0xee => Some((Instruction::INC, AddressMode::Absolute, 6)),
        0xef => None,

        0xf0 => Some((Instruction::BEQ, AddressMode::Relative, 2)),
        0xf1 => Some((Instruction::SBC, AddressMode::IndirectY, 4)),
        0xf2 => None,
        0xf3 => None,
        0xf4 => None,
        0xf5 => Some((Instruction::SBC, AddressMode::ZeropageX, 4)),
        0xf6 => Some((Instruction::INC, AddressMode::ZeropageX, 6)),
        0xf7 => None,
        0xf8 => Some((Instruction::SED, AddressMode::Implied, 2)),
        0xf9 => Some((Instruction::SBC, AddressMode::AbsoluteY, 4)),
        0xfa => None,
        0xfb => None,
        0xfc => None,
        0xfd => Some((Instruction::SBC, AddressMode::AbsoluteX, 4)),
        0xfe => Some((Instruction::INC, AddressMode::AbsoluteX, 7)),
        0xff => None,
    }
}

// The Instructions that the COU can execute
#[derive(Copy, Clone, Debug)]
enum Instruction {
    ADC, // add with carry
    AND, // and (with accumulator)
    ASL, // arithmetic shift left
    BCC, // branch on carry clear
    BCS, // branch on carry set
    BEQ, // branch on equal (zero set)
    BIT, // bit test
    BMI, // branch on minus (negative set)
    BNE, // branch on not equal (zero clear)
    BPL, // branch on plus (negative clear)
    BRK, // break / interrupt
    BVC, // branch on overflow clear
    BVS, // branch on overflow set
    CLC, // clear carry
    CLD, // clear decimal
    CLI, // clear interrupt disable
    CLV, // clear overflow
    CMP, // compare (with accumulator)
    CPX, // compare with X
    CPY, // compare with Y
    DEC, // decrement
    DEX, // decrement X
    DEY, // decrement Y
    EOR, // exclusive or (with accumulator)
    INC, // increment
    INX, // increment X
    INY, // increment Y
    JMP, // jump
    JSR, // jump subroutine
    LDA, // load accumulator
    LDX, // load X
    LDY, // load Y
    LSR, // logical shift right
    NOP, // no operation
    ORA, // or with accumulator
    PHA, // push accumulator
    PHP, // push processor status (SR)
    PLA, // pull accumulator
    PLP, // pull processor status (SR)
    ROL, // rotate left
    ROR, // rotate right
    RTI, // return from interrupt
    RTS, // return from subroutine
    SBC, // subtract with carry
    SEC, // set carry
    SED, // set decimal
    SEI, // set interrupt disable
    STA, // store accumulator
    STX, // store X
    STY, // store Y
    TAX, // transfer accumulator to X
    TAY, // transfer accumulator to Y
    TSX, // transfer stack pointer to X
    TXA, // transfer X to accumulator
    TXS, // transfer X to stack pointer
    TYA, // transfer Y to accumulator
}


#[cfg(test)]
pub mod tests {
    use crate::computer::memory::Memory;
    use super::*;

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

    #[test_log::test]
    fn creation() {
        let cpu = CPU::new(Memory::new(), TEST_ROM);

        assert_eq!(cpu.bus.read_address(RESET_ADDRESS), test_rom_start());
        assert_eq!(cpu.program_counter, test_rom_end_of_execution());
        assert_eq!(cpu.stack_pointer, 0xff);
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
