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

    pub fn read_byte(&self, address: u16) -> u8 {
		debug!(
			"[Read]\t\t{:02x} from {:04x}",
			self.data[address as usize], address
		);
		self.data[address as usize]
	}

    pub fn write_byte(&mut self, address: u16, value: u8) {
		debug!("[Write]\t\t{:02x} at {:04x}", value, address);
		self.data[address as usize] = value;
	}
    /* 
	fn read_word(&self, address: u16) -> u16 {
		let lower_byte = self.read_byte(address) as u16;
		let higher_byte = self.read_byte(address + 1) as u16;
		higher_byte << 8 | lower_byte
	}

	fn modify<F>(&mut self, address: u16, f: F)
	where
		F: Fn(u8) -> u8,
	{
		self.data[address as usize] = f(self.data[address as usize]);
	}
    */

}

#[cfg(test)]
mod tests {
    use std::mem;

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