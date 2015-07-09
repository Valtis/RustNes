use memory::Memory;
use rom::Mirroring;

use std::rc::Rc;
use std::cell::RefCell;

pub struct Vram {
    memory: Vec<u8>,
    mirroring: Mirroring,
}

impl Vram {
    pub fn new(mirroring: Mirroring, rom: Rc<RefCell<Box<Memory>>>) -> Vram {
        Vram {
            memory: vec![0;0x0800],
            mirroring: mirroring,
        }
    }

    fn get_nametable_address(&mut self, address: u16) -> usize {
        if address >= 0x2000 && address < 0x2400 { // nametable 0 does not need mirroring
            (address - 0x2000) as usize
        } else if address >= 0x2400 && address < 0x2800 {
            match self.mirroring {
                Mirroring::HorizontalMirroring => (address - 0x2400) as usize,
                Mirroring::VerticalMirroring => (address - 0x2400 + 0x400) as usize,
                _ => panic!("Invalid mirroring option when looking up nametable 1 address: {:?}", self.mirroring),
            }
        } else if address >= 0x2800 && address < 0x2C00 {
            match self.mirroring {
                Mirroring::HorizontalMirroring => (address - 0x2800 + 0x400) as usize,
                Mirroring::VerticalMirroring => (address - 0x2800) as usize,
                _ => panic!("Invalid mirroring option when looking up nametable 2 address: {:?}", self.mirroring),
            }
        }
        else if address >= 0x2C00 && address < 0x3000 { // nametable 3 does not need mirroring
            (address - 0x2C00 + 0x400) as usize
        } else {
            panic!("Invalid nametable address: 0x{:04X}", address);
        }
    }
}


impl Memory for Vram {
    fn read(&mut self, address: u16) -> u8 {
        if address >= 0x2000 && address < 0x3000 { // read from nametable
            let mem_address = self.get_nametable_address(address);
            self.memory[mem_address]
        } else {
            panic!("Read from PPU address 0x{:04X} is not implemented yet!", address);
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address >= 0x2000 && address < 0x3000 { // write to nametable
            let mem_address = self.get_nametable_address(address);
            self.memory[mem_address] = value;
        } else {
            panic!("Write to PPU address 0x{:04X} is not implemented yet!", address);
        }

    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;
    use rom::*;

    use std::rc::Rc;
    use std::cell::RefCell;

    struct MockMemory {
        memory: Vec<u8>
    }

    impl MockMemory {
        fn new() -> MockMemory {
            MockMemory {
                memory: vec![0;0xFFFF + 1],
            }
        }
    }

    impl Memory for MockMemory {
        fn read(&mut self, address: u16) -> u8 {
            self.memory[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.memory[address as usize] = value;
        }
    }

    fn create_test_vram() -> Vram {
        let ram = Rc::new(RefCell::new(Box::new(MockMemory::new()) as Box<Memory>));
        Vram::new(Mirroring::HorizontalMirroring, ram)
    }

    #[test]
    fn vram_writes_to_beginning_of_nametable_0_correctly() {
        let mut vram = create_test_vram();
        vram.write(0x2000, 0x12);
        assert_eq!(0x12, vram.memory[0x00]);
    }

    #[test]
    fn vram_writes_to_end_of_nametable_0_correctly() {
        let mut vram = create_test_vram();
        vram.write(0x23FF, 0x12);
        assert_eq!(0x12, vram.memory[0x03FF]);
    }

    #[test]
    fn vram_reads_from_beginning_of_nametable_0_correctly() {
        let mut vram = create_test_vram();
        vram.memory[0x00] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2000));
    }

    #[test]
    fn vram_reads_from_end_of_nametable_0_correctly() {
        let mut vram = create_test_vram();
        vram.memory[0x03FF] = 0xFE;
        assert_eq!(0xFE, vram.read(0x23FF));
    }

    #[test]
    fn vram_writes_to_beginning_of_nametable_1_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.write(0x2400, 0x12);
        assert_eq!(0x12, vram.memory[0x00]);
    }

    #[test]
    fn vram_writes_to_end_of_nametable_1_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.write(0x27FF, 0x12);
        assert_eq!(0x12, vram.memory[0x03FF]);
    }

    #[test]
    fn vram_reads_from_beginning_of_nametable_1_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.memory[0x00] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2400));
    }

    #[test]
    fn vram_reads_from_end_of_nametable_1_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.memory[0x03FF] = 0xFE;
        assert_eq!(0xFE, vram.read(0x27FF));
    }

    #[test]
    fn vram_writes_to_beginning_of_nametable_1_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.write(0x2400, 0x12);
        assert_eq!(0x12, vram.memory[0x400]);
    }

    #[test]
    fn vram_writes_to_end_of_nametable_1_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.write(0x27FF, 0x12);
        assert_eq!(0x12, vram.memory[0x07FF]);
    }

    #[test]
    fn vram_reads_from_beginning_of_nametable_1_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.memory[0x400] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2400));
    }

    #[test]
    fn vram_reads_from_end_of_nametable_1_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.memory[0x07FF] = 0xFE;
        assert_eq!(0xFE, vram.read(0x27FF));
    }


    #[test]
    fn vram_writes_to_beginning_of_nametable_2_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.write(0x2800, 0x12);
        assert_eq!(0x12, vram.memory[0x400]);
    }

    #[test]
    fn vram_writes_to_end_of_nametable_2_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.write(0x2BFF, 0x12);
        assert_eq!(0x12, vram.memory[0x07FF]);
    }

    #[test]
    fn vram_reads_from_beginning_of_nametable_2_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.memory[0x400] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2800));
    }

    #[test]
    fn vram_reads_from_end_of_nametable_2_correctly_with_horizontal_mirroring() {
        let mut vram = create_test_vram();
        vram.memory[0x07FF] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2BFF));
    }

    #[test]
    fn vram_writes_to_beginning_of_nametable_2_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.write(0x2800, 0x12);
        assert_eq!(0x12, vram.memory[0x000]);
    }

    #[test]
    fn vram_writes_to_end_of_nametable_2_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.write(0x2BFF, 0x12);
        assert_eq!(0x12, vram.memory[0x03FF]);
    }

    #[test]
    fn vram_reads_from_beginning_of_nametable_2_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.memory[0x000] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2800));
    }

    #[test]
    fn vram_reads_from_end_of_nametable_2_correctly_with_vertical_mirroring() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.memory[0x03FF] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2BFF));
    }

    #[test]
    fn vram_writes_to_beginning_of_nametable_3_correctly() {
        let mut vram = create_test_vram();
        vram.write(0x2C00, 0x12);
        assert_eq!(0x12, vram.memory[0x400]);
    }

    #[test]
    fn vram_writes_to_end_of_nametable_3_correctly() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.write(0x2FFF, 0x12);
        assert_eq!(0x12, vram.memory[0x07FF]);
    }

    #[test]
    fn vram_reads_from_beginning_of_nametable_3_correctly() {
        let mut vram = create_test_vram();
        vram.mirroring = Mirroring::VerticalMirroring;
        vram.memory[0x400] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2C00));
    }

    #[test]
    fn vram_reads_from_end_of_nametable_3_correctly() {
        let mut vram = create_test_vram();
        vram.memory[0x07FF] = 0xFE;
        assert_eq!(0xFE, vram.read(0x2FFF));
    }

}
