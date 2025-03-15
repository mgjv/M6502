use std::fmt::Debug;
use std::fmt;

use log::{trace, error};

// This function works in this order, because it's the order in which
// bytes are read from memory (i.e. little endian)
pub fn lo_hi_to_address(lo: u8, hi: u8) -> u16 {
    //u16::from(lo) + (u16::from(hi) << 8)
    u16::from_le_bytes([lo, hi])
}
pub fn bytes_to_address(bytes: &[u8]) -> u16 {
    lo_hi_to_address(bytes[0], bytes[1])
}
// Returns an array in little endian order, i.e. lo, hi
pub fn address_to_bytes(address: u16) -> [u8; 2] {
    address.to_le_bytes()
}

/*
 * Bus abstraction
 *
 * We should really get the address first and store it, then use
 * that address for the subsequent reads, but the situation where
 * a set is followed by multiple reads is much rarer than the set
 * and the read coming in pairs, so we optimise the API for that
*/
pub trait Bus: Debug {
    fn memory_size(&self) -> usize;

    fn read_byte(&self, address: u16) -> u8;

    fn read_two_bytes(&self, address: u16) -> [u8; 2] {
        if address >= 0xffff {
            error!("Attempt to read past end of memory");
        }
        [
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1)),
        ]
    }

    #[cfg(test)]
    fn read_four_bytes(&self, address: u16) -> [u8; 4] {
        if address >= 0xfffd {
            error!("Attempt to read past end of memory");
        }
        [
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1)),
            self.read_byte(address.wrapping_add(2)),
            self.read_byte(address.wrapping_add(3)),
        ]
    }

    fn write_byte(&mut self, address: u16, byte: u8);

    fn write_bytes(&mut self, start_address: u16, bytes: &[u8]) {
        // Default implementation probably should be overwritten
        let mut address = start_address;
        for b in bytes {
            self.write_byte(address, *b);
            address = address.wrapping_add(1);
        }
    }

    // Read the two bytes at given address, and return them as an address
    fn read_address(&self, address: u16) -> u16 {
        let b = self.read_two_bytes(address);
        lo_hi_to_address(b[0], b[1])
    }
}

// We will limit the address range from 0x0000 to 0xFFFF
const MAX_MEMORY_SIZE: usize = u16::MAX as usize + 1;
const DEFAULT_MEMORY_SIZE: usize = MAX_MEMORY_SIZE;

pub struct Memory {
    data: Vec <u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: vec!(0; DEFAULT_MEMORY_SIZE),
        }
    }
}

impl Bus for Memory {
    fn memory_size(&self) -> usize {
        self.data.len()
    }

    fn read_byte(&self, address: u16) -> u8 {
		trace!(
			"[Read]\t\t{:02x} from {:04x}",
			self.data[address as usize], address
		);
		self.data[address as usize]
	}

    fn write_byte(&mut self, address: u16, value: u8) {
		trace!("[Write]\t\t{:02x} at {:04x}", value, address);
		self.data[address as usize] = value;
	}

    fn write_bytes(&mut self, address: u16, bytes: &[u8]) {
        let offset = usize::from(address);
        self.data[offset..][..bytes.len()].copy_from_slice(bytes);
    }
}

// TODO somehow let the user determine how much and which memory to show
impl Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (rows, cols) = (5, 16);
        write!(f, "\"")?;
        for i in 0..rows {
            write!(f, "\n\t0x{:04X}:", i * cols)?;
            for j in 0..cols {
                if j % 8 == 0 {
                    write!(f, " ")?;
                }
                write!(f, " {:02X}", &self.data[i * cols + j])?;
            }
        }
        if rows * cols < self.memory_size() {
            write!(f, " ...")?;
        }
        write!(f, "\n\"")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn memory_creation() {
        let memory = Memory::new();
        // ensure memory is the correct size
        assert_eq!(MAX_MEMORY_SIZE, memory.memory_size());

        // By default memory should be initialised to 0
        for i in 0 .. MAX_MEMORY_SIZE - 1 {
            // let byte = memory.read_byte(i.into());
            let byte = memory.data[i];
            assert_eq!(byte, 0u8);
        }
    }

    #[test]
    fn memory_read_write() {
        let mut memory = Memory::new();
        for i in 0..0x100 {
            memory.write_byte(i, (i % 0xff) as u8);
        }
        for i in 0..0x100 {
            let byte = memory.read_byte(i);
            assert_eq!(byte, (i % 0xff) as u8);
        }

        let mut memory = Memory::new();

        let bytes = [0x10, 0x20, 0x30, 0x40, 0x50];
        memory.write_bytes(0x0000, &bytes);
        assert_eq!(memory.read_byte(0x0000), 0x10);
        assert_eq!(memory.read_byte(0x0001), 0x20);
        assert_eq!(memory.read_byte(0x0002), 0x30);
        assert_eq!(memory.read_byte(0x0003), 0x40);
        assert_eq!(memory.read_byte(0x0004), 0x50);
        assert_eq!(memory.read_byte(0x0005), 0x00);

        memory.write_bytes(0xffff - (bytes.len() - 1) as u16, &bytes);
        assert_eq!(memory.read_byte(0xffff), 0x50);
        assert_eq!(memory.read_byte(0xfffe), 0x40);
        assert_eq!(memory.read_byte(0xfffd), 0x30);
        assert_eq!(memory.read_byte(0xfffc), 0x20);
        assert_eq!(memory.read_byte(0xfffb), 0x10);
    }

    #[test]
    fn bytes_to_addr() {
        assert_eq!(0xdeadu16, lo_hi_to_address(0xad, 0xde));
        assert_eq!(0xbeefu16, lo_hi_to_address(0xef, 0xbe));
        assert_eq!(0x0000u16, lo_hi_to_address(0, 0));
        assert_eq!(0xffffu16, lo_hi_to_address(0xff, 0xff));
        assert_eq!(0xffffu16.wrapping_add(1), lo_hi_to_address(0, 0));
        assert_eq!(0x0000u16.wrapping_sub(1), lo_hi_to_address(0xff, 0xff));

        assert_eq!(address_to_bytes(0xdeadu16), [0xad, 0xde]);
        assert_eq!(address_to_bytes(0xbeefu16), [0xef, 0xbe]);
        assert_eq!(address_to_bytes(0x0000u16), [0, 0]);
        assert_eq!(address_to_bytes(0xffffu16), [0xff, 0xff]);
        assert_eq!(address_to_bytes(0xffffu16.wrapping_add(1)), [0, 0]);
        assert_eq!(address_to_bytes(0x0000u16.wrapping_sub(1)), [0xff, 0xff]);

        assert_eq!([0xde, 0xad], address_to_bytes(lo_hi_to_address(0xde, 0xad)));
    }
}
