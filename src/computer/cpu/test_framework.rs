use super::*;
use log::debug;
use std::fmt::Write;

pub const TEST_ROM: &'static[u8] = 
    &[ 0xa2, 0xff, 0x9a, 0x00, 0x00, 0x00, 0x00, 0x00,
       0x00, 0x00, 0x00, 0x00, 0xf0, 0xff, 0x00, 0x00 ];

pub fn test_rom_start() -> u16 {
    0xffff - (TEST_ROM.len() - 1) as u16
}

pub fn test_rom_end_of_execution () -> u16 {
    // find the first 0x00
    let offset = TEST_ROM.iter().position(|&b| b == 0x00).unwrap() as u16;
    test_rom_start() + offset as u16
}

#[derive(Debug, PartialEq, strum_macros::Display)]
enum TestOp {
    TestStart,
    TestEnd,

    TestA,
    TestX,
    TestY,

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

impl TestOp {
    fn debug_format(&self, bytes: &[u8; 4]) -> String {
        match self {
            TestOp::TestStart => format!("Test Start {:02x}", bytes[0]),
            TestOp::TestEnd => format!("Test End"),

            TestOp::TestA => format!("Test A == {:02x}", bytes[0]),
            TestOp::TestX => format!("Test X == {:02x}", bytes[0]),
            TestOp::TestY => format!("Test Y == {:02x}", bytes[0]),

            TestOp::TestCarrySet => format!("Test CarrySet"),
            TestOp::TestCarryClear => format!("Test CarryClear"),
            TestOp::TestZeroSet => format!("Test ZeroSet"),
            TestOp::TestZeroClear => format!("Test ZeroClear"),
            TestOp::TestNegativeSet => format!("Test NegativeSet"),
            TestOp::TestNegativeClear => format!("Test NegativeClear"),
            TestOp::TestOverflowSet => format!("Test OverflowSet"),
            TestOp::TestOverflowClear => format!("Test OverflowClear"),
            TestOp::TestDecimalSet => format!("Test DecimalSet"),
            TestOp::TestDecimalClear => format!("Test DecimalClear"),
            TestOp::TestInterruptSet => format!("Test InterruptSet"),
            TestOp::TestInterruptClear => format!("Test InterruptClear"),
            TestOp::TestBreakSet => format!("Test BreakSet"),
            TestOp::TestBreakClear => format!("Test BreakClear"),

            TestOp::TestAddressContents => 
                format!("Test AddressContents({:02x}{:02x}) = {:02x}", bytes[1], bytes[0], bytes[3]),
            TestOp::TestStackContents => 
                format!("Test StackContents({:02x}) = {:02x}", bytes[1], bytes[0]),
            TestOp::TestStackPointer => 
                format!("Test StackPointer = {:02x}", bytes[0]),
        }
    }

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

        //debug!("{:04x}: Verifying test with id {:02x}", start_address, test_id);

        let mut address = start_address;
        let mut op_num = 1;
        loop {
            let test_op_code = self.bus.read_byte(address);
            let test_op = TestOp::try_from(test_op_code).expect(
                &format!("Invalid test operation {:02x} at address {:04x}", test_op_code, address)
            );
            
            debug!("{:04x}:{:02x} (T {:02x}:{}) -> {}", 
                start_address, test_op_code, test_id, op_num,
                test_op.debug_format(&self.bus.read_four_bytes(address.wrapping_add(1))));

            match test_op {
                TestOp::TestStart => { address += 1; }, // only here for the debug line
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computer::memory::Memory;

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

}
