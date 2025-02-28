// use bitflags::bitflags;
use smart_default::SmartDefault;
use log::{debug, error};

use super::clock::TickCount;
use super::memory::Memory;

/*
 * For much of the information used here, see
 * https://www.masswerk.at/6502/6502_instruction_set.html
 */

// The CPU
#[derive(Debug, SmartDefault)]
pub struct CPU {
    // data_bus: u16,
    // address_bus: u16,

    // accumulator: u8,
    x_index: u8,
    y_index: u8,

    // stack_pointer: u16,
    program_counter: u16,
    // flags: u8,
}


// bitflags! {
//     pub struct Flags: u8 {
//         const PS_NEGATIVE           = 0b1000_0000;
//         const PS_OVERFLOW           = 0b0100_0000;
//         const PS_UNUSED             = 0b0010_0000; // JAM: Should this exist?
//                                                   // (note that it affects the
//                                                   // behavior of things like
//                                                   // from_bits_truncate)
//         const PS_BRK                = 0b0001_0000;
//         const PS_DECIMAL_MODE       = 0b0000_1000;
//         const PS_DISABLE_INTERRUPTS = 0b0000_0100;
//         const PS_ZERO               = 0b0000_0010;
//         const PS_CARRY              = 0b0000_0001;
//     }
// }

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    /* Run for one clock cycle */
    pub fn fetch_and_execute(&mut self, memory: &Memory) -> Option<TickCount> {
        // Read a byte
        let opcode = memory.read_byte(self.program_counter);

        // Identify the operator
        debug!("opcode {:02X} ", opcode);

        match decode_instruction(opcode) {
            // If we get a BRK, we halt execution
            Some((Instruction::BRK, _, _)) => {
                None
            }
            // Any other valid instruction, we process
            Some((instruction, address_mode, cycles)) => {
                
                // Fetch given arguments
                let operand_size = address_mode.operand_size();
                let operand_bytes = memory.read_two_bytes(self.program_counter + 1);
                let operand = self.get_operand(&memory, address_mode, operand_bytes);

                // update the state of memory and CPU
                self.execute(instruction, operand);

                // Advance the program counter by the correct number of bytes
                self.program_counter += 1 + operand_size;

                // FIXME This is here to stop us from running too long. Need to fix
                if self.program_counter > 10 {
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

    fn get_operand(&self, memory: &Memory, addressmode: AddressMode, bytes: [u8; 2]) -> Operand {
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
                let bytes = memory.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[0], bytes[1]))
            },
            AddressMode::IndirectX   => {
                // Add X to zero page address stored in bytes[0]. Return address stored there
                // TODO If bytes[0] is 0xfe, this is probably not right, as it will read past the zero page
                let address = bytes_to_address(0, bytes[0].wrapping_add(self.x_index));
                let bytes = memory.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[1], bytes[0]))
            },
            AddressMode::IndirectY   => {
                // Add contents of Y to address stored in zero page at byte[0] and byte[0] + 1, and return
                let address = bytes_to_address(0, bytes[0] );
                let bytes = memory.read_two_bytes(address);
                Operand::Address(bytes_to_address(bytes[1], bytes[0]).wrapping_add(self.y_index.into()))
            },
            // FIXME All of the below need to still be implemented
            AddressMode::Relative    => Operand::Implied,
            AddressMode::Zeropage    => Operand::Implied,
            AddressMode::ZeropageX   => Operand::Implied,
            AddressMode::ZeropageY   => Operand::Implied,
        }
    }

    fn execute(&mut self, instruction: Instruction, operand: Operand) {

    }
}

// Possible address modes for the above instructions
#[derive(Copy, Clone, Debug)]
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
enum Operand {
    Implied,
    Immediate(u8),
    Address(u16),
    Relative(u16),
}

// first byte is low, second high
fn bytes_to_address(lo: u8, hi: u8) -> u16 {
    u16::from(lo) + u16::from(hi) << 8
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
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let cpu = CPU::new();
        assert_eq!(cpu.program_counter, 0x0);
    }
}
