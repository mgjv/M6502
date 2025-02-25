use std::ops::Add;

use bitflags::bitflags;
use smart_default::SmartDefault;

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
    // x_index: u8,
    // y_index: u8,

    // stack_pointer: u16,
    program_counter: u16,
    // flags: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    /* Run for one clock cycle */
    pub fn fetch_and_execute(&mut self, memory: &Memory) -> Option<TickCount> {
        // Read a byte
        let opcode = memory.read_byte(self.program_counter);

        // Identify the operator
        print!("{:02X} ", opcode);

        // Let the operator fetch its arguments and update the state of memory and CPU
        // Advance the program counter by the correct number of bytes
        // return the number of cycles/ticks consumed

        self.program_counter += 1;

        if self.program_counter > 10 {
            return None;
        }
        Some(1)
    }

    // fn fetch(&self) {}

    // fn execute(&self) {}

    const fn decode_instruction(op_code: u8) -> Option<(Instruction, AddressMode)> {
        match op_code {
            0x00 => Some((Instruction::BRK, AddressMode::Implied)),
            0x01 => Some((Instruction::ORA, AddressMode::IndirectX)),
            0x02 => None,
            0x03 => None,
            0x04 => None,
            0x05 => Some((Instruction::ORA, AddressMode::Zeropage)),
            0x06 => Some((Instruction::ASL, AddressMode::Zeropage)),
            0x07 => None,
            0x08 => Some((Instruction::PHP, AddressMode::Implied)),
            0x09 => Some((Instruction::ORA, AddressMode::Immediate)),
            0x0a => Some((Instruction::ASL, AddressMode::Accumulator)),
            0x0b => None,
            0x0c => None,
            0x0d => Some((Instruction::ORA, AddressMode::Absolute)),
            0x0e => Some((Instruction::ASL, AddressMode::Absolute)),
            0x0f => None,

            0x10 => Some((Instruction::BPL, AddressMode::Relative)),
            0x11 => Some((Instruction::ORA, AddressMode::IndirectY)),
            0x12 => None,
            0x13 => None,
            0x14 => None,
            0x15 => Some((Instruction::ORA, AddressMode::ZeropageX)),
            0x16 => Some((Instruction::ASL, AddressMode::ZeropageX)),
            0x17 => None,
            0x18 => Some((Instruction::CLC, AddressMode::Implied)),
            0x19 => Some((Instruction::ORA, AddressMode::AbsoluteX)),
            0x1a => None,
            0x1b => None,
            0x1c => None,
            0x1d => Some((Instruction::ORA, AddressMode::AbsoluteX)),
            0x1e => Some((Instruction::ASL, AddressMode::AbsoluteX)),
            0x1f => None,

            0x20 => Some((Instruction::JSR, AddressMode::Absolute)),
            0x21 => Some((Instruction::AND, AddressMode::IndirectX)),
            0x22 => None,
            0x23 => None,
            0x24 => Some((Instruction::BIT, AddressMode::Zeropage)),
            0x25 => Some((Instruction::AND, AddressMode::Zeropage)),
            0x26 => Some((Instruction::ROL, AddressMode::Zeropage)),
            0x27 => None,
            0x28 => Some((Instruction::PLP, AddressMode::Implied)),
            0x29 => Some((Instruction::AND, AddressMode::Immediate)),
            0x2a => Some((Instruction::ROL, AddressMode::Accumulator)),
            0x2b => None,
            0x2c => Some((Instruction::BIT, AddressMode::Absolute)),
            0x2d => Some((Instruction::AND, AddressMode::Absolute)),
            0x2e => Some((Instruction::ROL, AddressMode::Absolute)),
            0x2f => None,

            0x30 => Some((Instruction::BMI, AddressMode::Relative)),
            0x31 => Some((Instruction::AND, AddressMode::IndirectY)),
            0x32 => None,
            0x33 => None,
            0x34 => None,
            0x35 => Some((Instruction::AND, AddressMode::ZeropageX)),
            0x36 => Some((Instruction::ROL, AddressMode::ZeropageX)),
            0x37 => None,
            0x38 => Some((Instruction::SEC, AddressMode::Implied)),
            0x39 => Some((Instruction::AND, AddressMode::AbsoluteY)),
            0x3a => None,
            0x3b => None,
            0x3c => None,
            0x3d => Some((Instruction::AND, AddressMode::AbsoluteX)),
            0x3e => Some((Instruction::ROL, AddressMode::AbsoluteX)),
            0x3f => None,

            0x40 => Some((Instruction::RTI, AddressMode::Implied)),
            0x41 => Some((Instruction::EOR, AddressMode::IndirectX)),
            0x42 => None,
            0x43 => None,
            0x44 => None,
            0x45 => Some((Instruction::EOR, AddressMode::Zeropage)),
            0x46 => Some((Instruction::LSR, AddressMode::Zeropage)),
            0x47 => None,
            0x48 => Some((Instruction::PHA, AddressMode::Implied)),
            0x49 => Some((Instruction::EOR, AddressMode::Immediate)),
            0x4a => Some((Instruction::LSR, AddressMode::Accumulator)),
            0x4b => None,
            0x4c => Some((Instruction::JMP, AddressMode::Absolute)),
            0x4d => Some((Instruction::EOR, AddressMode::Absolute)),
            0x4e => Some((Instruction::LSR, AddressMode::Absolute)),
            0x4f => None,

            0x50 => Some((Instruction::BVC, AddressMode::Relative)),
            0x51 => Some((Instruction::EOR, AddressMode::IndirectY)),
            0x52 => None,
            0x53 => None,
            0x54 => None,
            0x55 => Some((Instruction::EOR, AddressMode::ZeropageX)),
            0x56 => Some((Instruction::LSR, AddressMode::ZeropageX)),
            0x57 => None,
            0x58 => Some((Instruction::CLI, AddressMode::Implied)),
            0x59 => Some((Instruction::EOR, AddressMode::AbsoluteY)),
            0x5a => None,
            0x5b => None,
            0x5c => None,
            0x5d => Some((Instruction::EOR, AddressMode::AbsoluteX)),
            0x5e => Some((Instruction::LSR, AddressMode::AbsoluteX)),
            0x5f => None,

            0x60 => Some((Instruction::RTS, AddressMode::Implied)),
            0x61 => Some((Instruction::ADC, AddressMode::IndirectX)),
            0x62 => None,
            0x63 => None,
            0x64 => None,
            0x65 => Some((Instruction::ADC, AddressMode::Zeropage)),
            0x66 => Some((Instruction::ROR, AddressMode::Zeropage)),
            0x67 => None,
            0x68 => Some((Instruction::PLA, AddressMode::Implied)),
            0x69 => Some((Instruction::ADC, AddressMode::Immediate)),
            0x6a => Some((Instruction::ROR, AddressMode::Accumulator)),
            0x6b => None,
            0x6c => Some((Instruction::JMP, AddressMode::Indirect)),
            0x6d => Some((Instruction::ADC, AddressMode::Absolute)),
            0x6e => Some((Instruction::ROR, AddressMode::Absolute)),
            0x6f => None,

            0x70 => Some((Instruction::BVS, AddressMode::Relative)),
            0x71 => Some((Instruction::ADC, AddressMode::IndirectY)),
            0x72 => None,
            0x73 => None,
            0x74 => None,
            0x75 => Some((Instruction::ADC, AddressMode::Implied)),
            0x76 => Some((Instruction::ROR, AddressMode::Implied)),
            0x77 => None,
            0x78 => Some((Instruction::SEI, AddressMode::Implied)),
            0x79 => Some((Instruction::ADC, AddressMode::AbsoluteY)),
            0x7a => None,
            0x7b => None,
            0x7c => None,
            0x7d => Some((Instruction::ADC, AddressMode::AbsoluteX)),
            0x7e => Some((Instruction::ROR, AddressMode::AbsoluteX)),
            0x7f => None,

            0x80 => None,
            0x81 => Some((Instruction::STA, AddressMode::IndirectX)),
            0x82 => None,
            0x83 => None,
            0x84 => Some((Instruction::STY, AddressMode::Zeropage)),
            0x85 => Some((Instruction::STA, AddressMode::Zeropage)),
            0x86 => Some((Instruction::STX, AddressMode::Zeropage)),
            0x87 => None,
            0x88 => Some((Instruction::DEY, AddressMode::Implied)),
            0x89 => None,
            0x8a => Some((Instruction::TXA, AddressMode::Implied)),
            0x8b => None,
            0x8c => Some((Instruction::STY, AddressMode::Absolute)),
            0x8d => Some((Instruction::STA, AddressMode::Absolute)),
            0x8e => Some((Instruction::STX, AddressMode::Absolute)),
            0x8f => None,

            0x90 => Some((Instruction::BCC, AddressMode::Relative)),
            0x91 => Some((Instruction::STA, AddressMode::IndirectX)),
            0x92 => None,
            0x93 => None,
            0x94 => Some((Instruction::STY, AddressMode::ZeropageX)),
            0x95 => Some((Instruction::STA, AddressMode::ZeropageX)),
            0x96 => Some((Instruction::STX, AddressMode::ZeropageY)),
            0x97 => None,
            0x98 => Some((Instruction::TYA, AddressMode::Implied)),
            0x99 => Some((Instruction::STA, AddressMode::AbsoluteY)),
            0x9a => Some((Instruction::TXS, AddressMode::Implied)),
            0x9b => None,
            0x9c => None,
            0x9d => Some((Instruction::STA, AddressMode::AbsoluteX)),
            0x9e => None,
            0x9f => None,

            0xa0 => Some((Instruction::LDY, AddressMode::Immediate)),
            0xa1 => Some((Instruction::LDA, AddressMode::IndirectX)),
            0xa2 => Some((Instruction::LDX, AddressMode::Immediate)),
            0xa3 => None,
            0xa4 => Some((Instruction::LDY, AddressMode::Zeropage)),
            0xa5 => Some((Instruction::LDA, AddressMode::Zeropage)),
            0xa6 => Some((Instruction::LDX, AddressMode::Zeropage)),
            0xa7 => None,
            0xa8 => Some((Instruction::TAY, AddressMode::Implied)),
            0xa9 => Some((Instruction::LDA, AddressMode::Immediate)),
            0xaa => Some((Instruction::TAX, AddressMode::Implied)),
            0xab => None,
            0xac => Some((Instruction::LDY, AddressMode::Absolute)),
            0xad => Some((Instruction::LDA, AddressMode::Absolute)),
            0xae => Some((Instruction::LDX, AddressMode::Absolute)),
            0xaf => None,

            0xb0 => Some((Instruction::BCS, AddressMode::Relative)),
            0xb1 => Some((Instruction::LDA, AddressMode::IndirectY)),
            0xb2 => None,
            0xb3 => None,
            0xb4 => Some((Instruction::LDY, AddressMode::ZeropageX)),
            0xb5 => Some((Instruction::LDA, AddressMode::ZeropageX)),
            0xb6 => Some((Instruction::LDX, AddressMode::ZeropageY)),
            0xb7 => None,
            0xb8 => Some((Instruction::CLV, AddressMode::Implied)),
            0xb9 => Some((Instruction::LDA, AddressMode::Immediate)),
            0xba => Some((Instruction::TSX, AddressMode::Implied)),
            0xbb => None,
            0xbc => Some((Instruction::LDY, AddressMode::AbsoluteX)),
            0xbd => Some((Instruction::LDA, AddressMode::AbsoluteX)),
            0xbe => Some((Instruction::LDX, AddressMode::AbsoluteY)),
            0xbf => None,

            0xc0 => Some((Instruction::CPY, AddressMode::Immediate)),
            0xc1 => Some((Instruction::CMP, AddressMode::IndirectX)),
            0xc2 => None,
            0xc3 => None,
            0xc4 => Some((Instruction::CPY, AddressMode::Zeropage)),
            0xc5 => Some((Instruction::CMP, AddressMode::Zeropage)),
            0xc6 => Some((Instruction::DEC, AddressMode::Zeropage)),
            0xc7 => None,
            0xc8 => Some((Instruction::INY, AddressMode::Implied)),
            0xc9 => Some((Instruction::CMP, AddressMode::Immediate)),
            0xca => Some((Instruction::DEX, AddressMode::Implied)),
            0xcb => None,
            0xcc => Some((Instruction::CPY, AddressMode::Absolute)),
            0xcd => Some((Instruction::CMP, AddressMode::Absolute)),
            0xce => Some((Instruction::DEC, AddressMode::Absolute)),
            0xcf => None,

            0xd0 => Some((Instruction::BNE, AddressMode::Relative)),
            0xd1 => Some((Instruction::CMP, AddressMode::IndirectY)),
            0xd2 => None,
            0xd3 => None,
            0xd4 => None,
            0xd5 => Some((Instruction::CMP, AddressMode::ZeropageX)),
            0xd6 => Some((Instruction::DEC, AddressMode::ZeropageX)),
            0xd7 => None,
            0xd8 => Some((Instruction::CLD, AddressMode::Implied)),
            0xd9 => Some((Instruction::CMP, AddressMode::AbsoluteY)),
            0xda => None,
            0xdb => None,
            0xdc => None,
            0xdd => Some((Instruction::CMP, AddressMode::AbsoluteX)),
            0xde => Some((Instruction::DEC, AddressMode::AbsoluteX)),
            0xdf => None,

            0xe0 => Some((Instruction::CPX, AddressMode::Immediate)),
            0xe1 => Some((Instruction::SBC, AddressMode::IndirectX)),
            0xe2 => None,
            0xe3 => None,
            0xe4 => Some((Instruction::CPX, AddressMode::Zeropage)),
            0xe5 => Some((Instruction::SBC, AddressMode::Zeropage)),
            0xe6 => Some((Instruction::INC, AddressMode::Zeropage)),
            0xe7 => None,
            0xe8 => Some((Instruction::INX, AddressMode::Implied)),
            0xe9 => Some((Instruction::SBC, AddressMode::Immediate)),
            0xea => Some((Instruction::NOP, AddressMode::Implied)),
            0xeb => None,
            0xec => Some((Instruction::CPX, AddressMode::Absolute)),
            0xed => Some((Instruction::SBC, AddressMode::Absolute)),
            0xee => Some((Instruction::INC, AddressMode::Absolute)),
            0xef => None,

            0xf0 => Some((Instruction::BEQ, AddressMode::Relative)),
            0xf1 => Some((Instruction::SBC, AddressMode::IndirectY)),
            0xf2 => None,
            0xf3 => None,
            0xf4 => None,
            0xf5 => Some((Instruction::SBC, AddressMode::ZeropageX)),
            0xf6 => Some((Instruction::INC, AddressMode::ZeropageX)),
            0xf7 => None,
            0xf8 => Some((Instruction::BRK, AddressMode::Implied)),
            0xf9 => Some((Instruction::BRK, AddressMode::AbsoluteY)),
            0xfa => None,
            0xfb => None,
            0xfc => None,
            0xfd => Some((Instruction::BRK, AddressMode::Implied)),
            0xfe => Some((Instruction::BRK, AddressMode::Implied)),
            0xff => None,
        }
    }
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

impl AddressMode {
    // Returns the number of bytes needed for the operand
    const fn operand_size(&self) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let cpu = CPU::new();
        assert_eq!(cpu.program_counter, 0x0);
    }
}
