use std::fmt::Debug;
use std::fmt;

use log::{debug, error, info, trace};

// This function works in this order, because it's the order in which
// bytes are read from memory (i.e. little endian)
pub fn lo_hi_to_address(lo: u8, hi: u8) -> u16 {
    u16::from_le_bytes([lo, hi])
}
pub fn bytes_to_address(bytes: &[u8]) -> u16 {
    lo_hi_to_address(bytes[0], bytes[1])
}
// Returns an array in little endian order, i.e. lo, hi
pub fn address_to_bytes(address: u16) -> [u8; 2] {
    address.to_le_bytes()
}

#[derive(Debug)]
struct MappedAddressable {
    start: u16,
    end: u16,
    addressable: Box<dyn Addressable>,
}

#[derive(Debug)]
pub struct Bus {
    segments: Vec<MappedAddressable>,
}

/*
 * Bus
 *
 * The Bus represents the collection of all chips in the system connected to the
 * address and data buses. To build a bus, add segments (addressables) along with the
 * start address where you want them mapped in memory.
 *
 * If two address ranges of segments overlap, the last added segment is the one that will
 * be addressed
 */
impl Bus {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    // TODO addPartialRam?
    // TODO proper error when size goes past end of memeoiry range
    pub fn add_ram(self, ram: Ram, start: u16) -> Result<Self, String> {
        info!("Adding RAM of size {:x} at 0x{:04x}", ram.size(), start);
        let end = start + (ram.size() - 1) as u16;
        self.add_addressable(ram, start, end)
    }

    // TODO addPartialRom?
    // TODO proper error when size goes past end of memeoiry range
    pub fn add_rom(self, rom_data: &[u8], start: u16) -> Result<Self, String> {
        info!("Adding ROM of size {:x} at 0x{:04x}", rom_data.len(), start);
        let end = start +(rom_data.len() - 1) as u16;
        let rom = Ram::from(rom_data);
        self.add_addressable(rom, start, end)
    }

    pub fn add_rom_at_end(self, rom_data: &[u8]) -> Result<Self, String> {
        let start = MAX_MEMORY_SIZE - rom_data.len();
        self.add_rom(rom_data, start as u16)
    }

    fn add_addressable<A: Addressable + 'static>(mut self, addressable: A, start: u16, end: u16) -> Result<Self, String> {
        debug!("Adding addressable of size {:x} at 0x{:04x} to 0x{:04x}", addressable.size(), start, end);
        if start > end {
            return Err(format!("Start address 0x{:04x} is greater than end address 0x{:04x}", start, end));
        }
        if start % 0x100 != 0 || end % 0x100 != 0xff {
            return Err("Start and end must be aligned with page boundary".to_string());
        }

        let segment = MappedAddressable {
            start,
            end,
            addressable: Box::new(addressable),
        };
        // Insert at the front, so we don't have to iterate in reverse
        self.segments.insert(0, segment);
        Ok(self)
    }
}

impl Addressable for Bus {

    fn read_byte(&self, address: u16) -> u8 {
        for segment in &self.segments {
            if address >= segment.start && address <= segment.end {
                return segment.addressable.read_byte(address - segment.start);
            }
        }
        error!("Attempt to read from unmapped memory address 0x{:04x}", address);
        0
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        for segment in &mut self.segments {
            if address >= segment.start && address <= segment.end {
                segment.addressable.write_byte(address - segment.start, byte);
                return;
            }
        }
        error!("Attempt to write to unmapped memory address 0x{:04x}", address);
    }

    // TODO implement read_two and four bytes more efficiently

    // TODO Is this what we want here? Do we want to return the sum of RAM?
    fn size(&self) -> usize {
        MAX_MEMORY_SIZE
    }
}

/*
 * Bus abstraction
 *
 * We should really get the address first and store it, then use
 * that address for the subsequent reads, but the situation where
 * a set is followed by multiple reads is much rarer than the set
 * and the read coming in pairs, so we optimise the API for that
*/
pub trait Addressable: Debug {
    fn size(&self) -> usize;

    fn read_byte(&self, address: u16) -> u8;

    fn read_two_bytes(&self, address: u16) -> [u8; 2] {
        if address == 0xffff {
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

// TODO add a 'proper' bus implementation with multiple rom and ram regions
// and special handling of addresses where needed

// We will limit the address range from 0x0000 to 0xFFFF
pub const MAX_MEMORY_SIZE: usize = u16::MAX as usize + 1;
const DEFAULT_MEMORY_SIZE: usize = MAX_MEMORY_SIZE;

#[derive(Clone)]
pub struct Ram {
    data: Vec <u8>,
}

impl Default for Ram {
    fn default() -> Self {
        Self::new(DEFAULT_MEMORY_SIZE)
    }
}

impl Ram {
    // TODO Do we need to enforce page size multiple?
    pub fn new(size: usize) -> Self {
        Self {
            data: vec!(0; size),
        }
    }

    // TODO this is really a Rom thing...
    pub fn from(data: &[u8]) -> Self {
        Self {
            data: Vec::from(data),
        }
    }
}

impl Addressable for Ram {
    fn size(&self) -> usize {
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
// TODO can't I move this to Addressable?
// TODO Why do I even need this?
impl Debug for Ram {
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

// Do nothing implementation of a Bus to allow chios to 'float'
#[derive(Debug)]
pub struct UnconnectedBus {}

impl Addressable for UnconnectedBus {
    fn size(&self) -> usize {
        0
    }

    fn read_byte(&self, _address: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, _address: u16, _byte: u8) {
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn ram_creation() {
        let memory = Ram::default();
        // ensure memory is the correct size
        assert_eq!(MAX_MEMORY_SIZE, memory.size());

        // By default memory should be initialised to 0
        for i in 0 .. MAX_MEMORY_SIZE - 1 {
            // let byte = memory.read_byte(i.into());
            let byte = memory.data[i];
            assert_eq!(byte, 0u8);
        }
    }

    #[test]
    fn memory_read_write() {
        let mut memory = Ram::new(0x100);
        for i in 0..0x100 {
            memory.write_byte(i, (i % 0xff) as u8);
        }
        for i in 0..0x100 {
            let byte = memory.read_byte(i);
            assert_eq!(byte, (i % 0xff) as u8);
        }

        let mut memory = Ram::default();

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

    #[test]
    fn test_unconnected_bus() {
        let mut bus = UnconnectedBus{};
        assert_eq!(bus.size(), 0);
        assert_eq!(bus.read_byte(0), 0);
        bus.write_byte(0, 0);
        bus.write_byte(0xffff, 0);
    }

    impl Bus {
        fn get_segment_at_start_address(&self, address: u16) -> Option<&dyn Addressable> {
            for segment in &self.segments {
                if address == segment.start {
                    return Some(&*segment.addressable);
                }
            }
            None
        }
    }

    #[test]
    #[should_panic(expected = "Start and end must be aligned")]
    fn unaligned_ram_size() {
        Bus::new()
            .add_ram(Ram::new(0x150), 0x00)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Start and end must be aligned")]
    fn unaligned_start_address() {
        Bus::new()
            .add_ram(Ram::new(0x200), 0x0050)
            .unwrap();
    }

    #[test]
    fn test_overlap_bus() -> Result<(), String> {
        let mut ram1 = Ram::new(0x200);
        for i in 0..0x200 {
            ram1.write_byte(i, 0x01);
        }
        let mut ram2 = Ram::new(0x200);
        for i in 0..0x200 {
            ram2.write_byte(i, 0x02);
        }
        // Create a bus where the two segments overlap
        let mut bus = Bus::new()
            .add_ram(ram1, 0x0000)?
            .add_ram(ram2, 0x0100)?;

        assert_eq!(0x01, bus.read_byte(0x0000));
        assert_eq!(0x01, bus.read_byte(0x00ff));
        assert_eq!(0x02, bus.read_byte(0x0100));
        assert_eq!(0x02, bus.read_byte(0x01ff));

        bus.write_byte(0x0000, 0xff);
        bus.write_byte(0x00ff, 0xff);
        bus.write_byte(0x0100, 0xff);
        bus.write_byte(0x01ff, 0xff);

        // During testing we can pull out a reference to the segments
        let ram1 = bus.get_segment_at_start_address(0x0000).unwrap();
        let ram2 = bus.get_segment_at_start_address(0x0100).unwrap();

        assert_eq!(0xff, ram1.read_byte(0x0000));
        assert_eq!(0xff, ram1.read_byte(0x00ff));
        assert_eq!(0x01, ram1.read_byte(0x0100));
        assert_eq!(0x01, ram1.read_byte(0x01ff));
        assert_eq!(0xff, ram2.read_byte(0x0000));
        assert_eq!(0xff, ram2.read_byte(0x00ff));
        assert_eq!(0x02, ram2.read_byte(0x0100));
        assert_eq!(0x02, ram2.read_byte(0x01ff));

        Ok(())
    }

    #[test]
    fn test_rom() -> Result<(), String> {
        // Create some fake rom images
        let test_rom1: Vec<u8> = (0..0x200).map(|it| (it % 0x100) as u8).collect();
        let test_rom2: Vec<u8> = test_rom1.iter().rev().copied().collect();

        // Load one at the start, one in the middle somewhere and one at the end
        let bus = Bus::new()
            .add_rom(&test_rom1, 0x0000)?
            .add_rom(&test_rom1, 0x1000)?
            .add_rom_at_end(&test_rom2)?;

        assert_eq!(0x00, bus.read_byte(0x0000));
        assert_eq!(0x01, bus.read_byte(0x0001));
        assert_eq!(0xff, bus.read_byte(0x01ff));

        assert_eq!(0x00, bus.read_byte(0x1000));
        assert_eq!(0x01, bus.read_byte(0x1001));
        assert_eq!(0xff, bus.read_byte(0x11ff));

        assert_eq!(0xff, bus.read_byte(0xfe00));
        assert_eq!(0x00, bus.read_byte(0xffff));

        // TODO Roms should not be writable

        Ok(())
    }
}
