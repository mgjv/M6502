use std::fmt;

use log::debug;

// We will limit the address range from 0x0000 to 0xFFFF
// TODO The fact that this needs to be a usize, but all our addressing 
//      is in u16 is a bit of a pain.
const MAX_MEMORY_SIZE: usize = u16::MAX as usize + 1;

pub struct Memory {
    data: Vec <u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        assert!(size <= MAX_MEMORY_SIZE);

        Self {
            data: vec!(0; usize::from(size)),
        }
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }

    // TODO do we need to make this "safe" for end of memory space? 
    pub fn read_byte(&self, address: u16) -> u8 {
		debug!(
			"[Read]\t\t{:02x} from {:04x}",
			self.data[address as usize], address
		);
		self.data[address as usize]
	}

    // Read the next two bytes from memory, and return it as an address
    // TODO do we need to make this "safe" for end of memory space? 
    pub fn read_two_bytes(&self, address: u16) -> [u8; 2] {
        // TODO this could probably be done with a slice?
        [
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1)),
        ]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
		debug!("[Write]\t\t{:02x} at {:04x}", value, address);
		self.data[address as usize] = value;
	}
}

// TODO somehow let the user determine how much and which memory to show
impl fmt::Debug for Memory {
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
        if rows * cols < self.size() {
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
    fn creation() {
        let memory = Memory::new(MAX_MEMORY_SIZE);
        // ensure memory is the correct size
        assert_eq!(MAX_MEMORY_SIZE, memory.size());

        let memory = Memory::new(0x100);
        // ensure memory is the correct size
        assert_eq!(0x100, memory.size());
        // By default memory should be initialised to 0
        for i in 0..0x100 {
            let byte = memory.read_byte(i);
            assert_eq!(byte, 0u8);
        }

        let result = std::panic::catch_unwind(|| {
            // Allocating more memory than allowed should error out
            Memory::new(MAX_MEMORY_SIZE + 1);
        });
        assert!(result.is_err());
    }

    #[test]
    fn reading_and_writing() {
        let mem_size = 0x200;
        let mut memory = Memory::new(mem_size);
        for i in 0..0x100 {
            memory.write_byte(i, (i % 0xff) as u8);
        }
        for i in 0..0x100 {
            let byte = memory.read_byte(i);
            assert_eq!(byte, (i % 0xff) as u8);
        }
    }
}