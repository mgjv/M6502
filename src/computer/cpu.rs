mod inspect;
mod instruction;
use instruction::*;
mod status;
use status::*;

use log::{debug, error};
use inline_colorization::*;

use std::fmt;

use super::clock::TickCount;
use super::bus::*;


// Standard memory locations to fetch addresses from
const NMI_ADDRESS: u16 = 0xfffa;
const RESET_ADDRESS: u16 = 0xfffc;
const IRQ_ADDRESS: u16 = 0xfffe;

/*
 * For much of the information used here, see
 * https://www.masswerk.at/6502/6502_instruction_set.html
 */

// The CPU
#[derive(Debug)]
pub struct Cpu {
    pub bus: Bus,

    accumulator: u8,
    x_index: u8,
    y_index: u8,

    // Stack is on page 1, $0100 - $01ff, runs high -> low
    stack_pointer: u8,
    program_counter: u16,
    status: Status,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {

        Self {
            bus,
            accumulator: 0,
            x_index: 0,
            y_index: 0,

            stack_pointer: 0xfd,

            program_counter: RESET_ADDRESS,
            status: Status::default(),
        }
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

        match decode_instruction(opcode) {
            // FIXME This isn't the right implementation for BRK
            Some((Instruction::BRK, _, _)) => {
                debug!("{:04x}:{:02x} -> BRK  -> BRK", self.program_counter, opcode);
                self.execute_instruction(Instruction::BRK, Operand::Implied);
                None
            },
            #[cfg(test)]
            Some((Instruction::HALT, _, _)) => {
                debug!("{:04x}:{:02x} -> HALT", self.program_counter, opcode);
                None
            },
            #[cfg(test)]
            #[allow(clippy::assertions_on_constants)]
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
                // update cycles with extra if page boundaries are crossed
                let cycles = cycles + self.get_extra_cyles(address_mode, operand_bytes);

                debug!("{:04x}:{:02x} -> {} {} -> {} {}",
                    self.program_counter, opcode,
                    instruction, address_mode.debug_format(operand_bytes),
                    instruction, operand.debug_format());

                // Advance the program counter by the correct number of bytes
                // This is done before the instruction is executed, so that the instruction can
                // modify the program counter if needed, without running the risk that this
                // overwrites it again
                self.program_counter += 1 + operand_size;

                // update the state of memory and CPU
                self.execute_instruction(instruction, operand);

                // FIXME This is here to stop us from running too long. Need to fix
                if self.program_counter > NMI_ADDRESS {
                    error!("Program counter reached {:04X}, halting", self.program_counter);
                    return None;
                }

                // return the number of cycles/ticks consumed
                Some(cycles as TickCount)
            },
            // Hmm, this is not  valid instruction. Wonder what happened
            None => {
                // This shouldn't happen
                error!("{:04x}: Unused opcode {:02x} found", self.program_counter, opcode);
                // TODO Decide what should happen here in production code. panic is fine for test
                panic!()
            },
        }
    }

    fn get_extra_cyles(&self, address_mode: AddressMode, bytes: [u8; 2]) -> u8 {
        match address_mode {
            AddressMode::AbsoluteX => {
                let address = bytes_to_address(&bytes).wrapping_add(self.x_index.into());
                self.crosses_page_boundary(address) as u8
            },
            AddressMode::AbsoluteY => {
                let address = bytes_to_address(&bytes).wrapping_add(self.y_index.into());
                self.crosses_page_boundary(address) as u8
            },
            AddressMode::IndirectY => {
                // Get the address of the zero page given
                let address = lo_hi_to_address(bytes[0], 0x00);
                // Read the actual address stored at the given address and offset by Y
                let address = bytes_to_address(&self.bus.read_two_bytes(address))
                    .wrapping_add(self.y_index.into());
                self.crosses_page_boundary(address) as u8
            }
            AddressMode::Relative => {
                // offset is a 2's complement signed byte
                let offset = bytes[0] as i8;
                // offset is relative to immediate next instruction address
                let address = self.program_counter.wrapping_add(2).wrapping_add(offset as u16);

                1 + self.crosses_page_boundary(address) as u8
            }

            _ => 0,
        }
    }

    fn get_operand(&self, addressmode: AddressMode, bytes: [u8; 2]) -> Operand {
        match addressmode {
            AddressMode::Accumulator => Operand::Implied,
            AddressMode::Absolute    => {
                let address = bytes_to_address(&bytes);
                Operand::Address(address)
            },
            AddressMode::AbsoluteX   => {
                let address = bytes_to_address(&bytes).wrapping_add(self.x_index.into());
                Operand::Address(address)
            },
            AddressMode::AbsoluteY   => {
                let address = bytes_to_address(&bytes).wrapping_add(self.y_index.into());
                Operand::Address(address)
            },
            AddressMode::Immediate   => Operand::Immediate(bytes[0]),
            AddressMode::Implied     => Operand::Implied,
            AddressMode::Indirect    => {
                let address = bytes_to_address(&bytes);
                // Read the actual address stored at the givenm address
                let address = bytes_to_address(&self.bus.read_two_bytes(address));
                Operand::Address(address)
            },
            AddressMode::IndirectX   => {
                // Add X to zero page address stored in bytes[0].
                let address = lo_hi_to_address(bytes[0].wrapping_add(self.x_index), 0x00);
                // Read the actual address stored at the given address
                let address = bytes_to_address(&self.bus.read_two_bytes(address));
                Operand::Address(address)
            },
            AddressMode::IndirectY   => {
                // Get the address of the zero page given
                let address = lo_hi_to_address(bytes[0], 0x00);
                // Read the actual address stored at the given address and offset by Y
                let address = bytes_to_address(&self.bus.read_two_bytes(address))
                    .wrapping_add(self.y_index.into());
                Operand::Address(address)
            },
            AddressMode::Relative    => {
                // offset is a 2's complement signed byte
                let offset = bytes[0] as i8;
                // offset is relative to immediate next instruction address
                let address = self.program_counter.wrapping_add(2).wrapping_add(offset as u16);
                Operand::Address(address)
            },
            AddressMode::Zeropage    => {
                let address = lo_hi_to_address(bytes[0], 0);
                Operand::Address(address)
            },
            AddressMode::ZeropageX   => {
                let address = lo_hi_to_address(bytes[0], 0).wrapping_add(self.x_index.into());
                Operand::Address(address)
            },
            AddressMode::ZeropageY   => {
                let address = lo_hi_to_address(bytes[0], 0).wrapping_add(self.y_index.into());
                Operand::Address(address)
            },
        }
    }

    fn crosses_page_boundary(&self, address: u16) -> bool {
        let pc_page = self.program_counter & 0xff00;
        let address_page = address & 0xff00;
        pc_page != address_page
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
            Instruction::BIT => {
                match operand {
                    Operand::Address(address) => {
                        let value = self.bus.read_byte(address);
                        self.status.zero = self.accumulator & value == 0;
                        self.status.overflow = value & 0x40 != 0;
                        self.status.negative = value & 0x80 != 0;
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
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
            Instruction::BRK => {
                self.execute_brk();
            },
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
            Instruction::CLC => {
                self.status.carry = false;
            },
            Instruction::CLD => {
                self.status.decimal = false;
            },
            Instruction::CLI => {
                self.status.irq_disable = false;
            },
            Instruction::CLV => {
                self.status.overflow = false;
            },
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
            Instruction::INX => {
                self.set_x_index(self.x_index.wrapping_add(1))
            },
            Instruction::INY => {
                self.set_y_index(self.y_index.wrapping_add(1))
            },
            Instruction::JMP => {
                self.do_jump(instruction, operand);
            },
            Instruction::JSR => {
                match operand {
                    Operand::Address(address) => {
                        // program counter already points to next instruction
                        let return_address = self.program_counter;
                        let bytes = address_to_bytes(return_address);
                        self.push_stack(bytes[0]);
                        self.push_stack(bytes[1]);
                        self.program_counter = address;
                    },
                    _ => illegal_opcode(instruction, operand),
                }
            },
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
            Instruction::NOP => { }, // doesn't do anything
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
            Instruction::PHA => {
                self.push_stack(self.accumulator);
            },
            Instruction::PHP => {
                // PHP pushes the status register on the stack with bits 4 and 5 set
                let mut status = self.status;
                status.brk = true;
                status.ignored = true;
                self.push_stack(status.as_byte());
            },
            Instruction::PLA => {
                let value = self.pull_stack();
                self.set_accumulator(value);
            },
            Instruction::PLP => {
                // PLP sets bits 7, 6, 4, 3, 2, and 1 of the status register
                // to the value on the stack
                let value = self.pull_stack();
                let mut new_status = Status::from_byte(value);
                // Bits 4 (brk) and 5 (ignored) are not affected
                new_status.ignored = self.status.ignored;
                new_status.brk = self.status.brk;
                self.status = new_status;
                debug!("PLP: setting status %{:08b}", self.status.as_byte());
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
            Instruction::RTI => {
                self.return_from_interrupt();
            },
            Instruction::RTS => {
                let hi = self.pull_stack();
                let lo = self.pull_stack();
                let address = lo_hi_to_address(lo, hi);
                self.program_counter = address;
            },
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
            Instruction::SEC => {
                self.status.carry = true;
            },
            Instruction::SED => {
                self.status.decimal = true;
            },
            Instruction::SEI => {
                self.status.irq_disable = true;
            },
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
            Instruction::TAX => {
                self.x_index = self.accumulator;
            },
            Instruction::TAY => {
                self.y_index = self.accumulator;
            },
            Instruction::TSX => {
                self.x_index = self.stack_pointer;
            },
            Instruction::TXA => {
                self.accumulator = self.x_index;
            },
            Instruction::TXS => {
                self.stack_pointer = self.x_index;
            },
            Instruction::TYA => {
                self.accumulator = self.y_index;
            },
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
                panic!("{} should already have been handled before this", instruction);
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
        self.program_counter = lo_hi_to_address(high, low);
        self.status = Status::from_byte(status);
        // Ensure brk is always false after a hardware restore
        self.status.brk = false;
    }

    // Push a new value on the stack, decrementing the stack pointer
    fn push_stack(&mut self, value: u8) {
        let address = lo_hi_to_address(self.stack_pointer, 0x01);
        self.bus.write_byte(address, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    // Pull the 'top' value off the stack, incrementing the stack pointer
    fn pull_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let address = lo_hi_to_address(self.stack_pointer, 0x01);
        self.bus.read_byte(address)
    }

    #[cfg(test)]
    // get the value on the stack at position n, where n = 1 is the 'top'
    fn peek_stack(&self, position: u8) -> u8 {
        let stack_position = self.stack_pointer.wrapping_add(position);
        let address = lo_hi_to_address(stack_position, 0x01);
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

// FIXME change this. Might need to move to instruction, but probably needs to be
// split into one for CPU and one for Instruction
fn illegal_opcode(instruction: Instruction, operand: Operand) {
    error!("Attempt to execute illegal opcode for {:?} with operand {:?}", instruction, operand);
}

#[cfg(test)]
mod test_framework;

#[cfg(test)]
pub mod tests {
    use crate::computer::bus::Ram;
    use super::*;
    use super::test_framework::*;

    use test_log::test;

    fn create_test_cpu() -> Cpu {
        let bus = Bus::new()
        .add_ram(Ram::default(), 0x0).unwrap()
        .add_rom_at_end(&test_rom()).unwrap();
        Cpu::new(bus)
    }

    #[test]
    fn creation() {
        let cpu = create_test_cpu();
        assert_eq!(cpu.bus.read_address(RESET_ADDRESS), test_rom_start_of_data());

        // TODO What else?
    }

    #[test]
    fn load_program() {
        let mut cpu = create_test_cpu();

        let program = [0xa9, 0x01, 0x69, 0x02, 0x8d, 0x02];
        let load_address = 0xc000;

        cpu.load_program(load_address, &program);

        for (i, &byte) in program.iter().enumerate() {
            let data = cpu.bus.read_byte(load_address + i as u16);
            assert_eq!(byte, data);
        }
    }
}
