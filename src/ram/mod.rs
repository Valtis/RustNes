use std::fmt;
// Badly unfinished. Consider this to be a placeholder for now.

use memory::Memory;


pub struct Ram {
    memory: Vec<u8>,
}

impl fmt::Debug for Ram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Ram content not shown>")
    }
}

impl Memory for Ram {
    fn read(&self, address: u16) -> u8 {
        // ram mirroring
        if address < 0x2000 {
            self.memory[(address & 0x07FF) as usize]
        } else {
            panic!("Read from non-existent ram address 0x{:04X}", address);
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            self.memory[(address & 0x07FF) as usize] = value;
        } else {
            panic!("Write to non-existent ram address 0x{:04X}", address);
        }
    }
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            memory: vec![0;0x0800],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::*;

    fn create_test_memory() -> Ram {
        Ram::new()
    }

    #[test]
    fn write_to_memory_ram_address_writes_data_correctly() {
        let mut ram = create_test_memory();
        ram.write(0x0520, 0xAF);
        assert_eq!(0xAF, ram.memory[0x0520]);
    }

    #[test]
    fn read_from_ram_address_returns_correct_data() {
        let mut ram = create_test_memory();
        ram.memory[0x0520] = 0xAF;
        assert_eq!(0xAF, ram.read(0x0520));
    }

    #[test]
    fn write_to_mirrored_address_writes_to_correct_ram_address() {
        let mut ram = create_test_memory();
        ram.write(0x0B12, 0xAF);
        assert_eq!(0xAF, ram.memory[0x0312]);
    }

    #[test]
    fn read_from_mirrored_section_returns_correct_data() {
        let mut ram = create_test_memory();
        ram.memory[0x312] = 0xAF;
        assert_eq!(0xAF, ram.read(0x0B12));
    }

    // edge case tests
    #[test]
    fn write_to_final_mirrored_ram_address_writes_to_correct_address() {
        let mut ram = create_test_memory();
        ram.write(0x1FFF, 0xAF);
        assert_eq!(0xAF, ram.memory[0x07FF]);
    }

    #[test]
    fn read_from_final_ram_address_returns_correct_data() {
        let mut ram = create_test_memory();
        ram.memory[0x07FF] = 0xAF;
        assert_eq!(0xAF, ram.read(0x1FFF));
    }

    #[test]
    #[should_panic]
    fn read_from_above_0x1FFF_panics() {
        let mut ram = create_test_memory();
        ram.read(0x2000);
    }

    #[test]
    #[should_panic]
    fn write_from_above_0x1FFF_panics() {
        let mut ram = create_test_memory();
        ram.read(0x3234);
    }
}
