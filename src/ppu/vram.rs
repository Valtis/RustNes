use memory::Memory;
use rom::Mirroring;

use std::rc::Rc;
use std::cell::RefCell;

pub struct Vram {
    rom: Rc<RefCell<Box<Memory>>>,
    memory: Vec<u8>, // regular 2kb ram
    palette_memory: Vec<u8>, // memory for palettes, 32 bytes
    mirroring: Mirroring,
}

impl Vram {
    pub fn new(mirroring: Mirroring, rom: Rc<RefCell<Box<Memory>>>) -> Vram {
        Vram {
            rom: rom,
            memory: vec![0;0x0800],
            palette_memory: vec![0;0x20],
            mirroring: mirroring,
        }
    }

    // calculates address to ppu ram from ppu memory map address
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
        } else if address >= 0x3000 && address < 0x3F00 { // 0x3000 - 0x3EFFF is mirror of 0x2000 - 0x2EFF
            self.get_nametable_address(address - 0x1000)
        }
        else {
            panic!("Invalid nametable address: 0x{:04X}", address);
        }
    }

    // calculates address to ppu palette memory from ppu memory map address
    fn get_palette_address(&mut self, address: u16) -> usize {
        let masked_address = address & 0x001F; // mask out the ignored bits
        match masked_address {
            0x0010 => 0x0000, // mirrored addresses
            0x0014 => 0x0004,
            0x0018 => 0x0008,
            0x001C => 0x000C,
            _ => masked_address as usize,
        }
    }
}


impl Memory for Vram {
    fn read(&mut self, address: u16) -> u8 {
        if address < 0x2000 {
            self.rom.borrow_mut().read(address)
        } else if address >= 0x2000 && address < 0x3F00 { // read from nametable
            let mem_address = self.get_nametable_address(address);
            self.memory[mem_address]
        } else if address >= 0x3F00 && address <= 0x3FFF { // read from palette memory
            let palette_address = self.get_palette_address(address);
            self.palette_memory[palette_address]
        } else {
            panic!("Read from PPU address 0x{:04X} is not implemented yet!", address);
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            self.rom.borrow_mut().write(address, value);
        } else if address >= 0x2000 && address < 0x3F00 { // write to nametable
            let mem_address = self.get_nametable_address(address);
            self.memory[mem_address] = value;
        } else if address >= 0x3F00 && address <= 0x3FFF { // write to palette memory
            let palette_address = self.get_palette_address(address);
            self.palette_memory[palette_address] = value;
        }  else {
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
        let rom = Rc::new(RefCell::new(Box::new(MockMemory::new()) as Box<Memory>));
        Vram::new(Mirroring::HorizontalMirroring, rom)
    }

    #[test]
    fn write_to_0x0000_is_redirected_to_rom() {
        let mut vram = create_test_vram();
        vram.write(0x0000, 0x7B);
        assert_eq!(0x7B, vram.rom.borrow_mut().read(0x0000));
    }

    #[test]
    fn read_from_0x0000_is_redirected_to_rom() {
        let mut vram = create_test_vram();
        vram.rom.borrow_mut().write(0x0000, 0x7B);
        assert_eq!(0x7B, vram.read(0x0000));
    }

    #[test]
    fn write_to_0x1FFF_is_redirected_to_rom() {
        let mut vram = create_test_vram();
        vram.write(0x1FFF, 0x7B);
        assert_eq!(0x7B, vram.rom.borrow_mut().read(0x1FFF));
    }

    #[test]
    fn read_from_0x1FFF_is_redirected_to_rom() {
        let mut vram = create_test_vram();
        vram.rom.borrow_mut().write(0x1FFF, 0x7B);
        assert_eq!(0x7B, vram.read(0x1FFF));
    }

    #[test]
    fn write_to_0x2000_is_not_redirected_to_rom() {
        let mut vram = create_test_vram();
        vram.write(0x2000, 0x7B);
        assert_eq!(0x00, vram.rom.borrow_mut().read(0x2000));
    }

    #[test]
    fn read_from_0x2000_is_not_redirected_to_rom() {
        let mut vram = create_test_vram();
        vram.rom.borrow_mut().write(0x2000, 0x7B);
        assert_eq!(0x00, vram.read(0x2000));
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

    #[test]
    fn vram_address_0x3000_mirrors_to_0x2000() {
        let mut vram = create_test_vram();
        vram.write(0x2000, 0xFE);
        assert_eq!(0xFE, vram.read(0x3000));
    }

    #[test]
    fn vram_address_0x3500_mirrors_to_0x2500() {
        let mut vram = create_test_vram();
        vram.write(0x2500, 0xFE);
        assert_eq!(0xFE, vram.read(0x3500));
    }

    #[test]
    fn vram_address_0x3EFF_mirrors_to_0x2EFF() {
        let mut vram = create_test_vram();
        vram.write(0x2EFF, 0xFE);
        assert_eq!(0xFE, vram.read(0x3EFF));
    }

    #[test]
    fn write_to_vram_address_0x3F00_writes_to_beginning_of_palette_ram() {
        let mut vram = create_test_vram();
        vram.write(0x3F00, 0xFE);
        assert_eq!(0xFE, vram.palette_memory[0x00]);
    }


    #[test]
    fn read_from_vram_address_0x3F00_reads_from_beginning_of_palette_ram() {
        let mut vram = create_test_vram();
        vram.palette_memory[0x00] = 0x15;
        assert_eq!(0x15, vram.read(0x3F00));
    }

    #[test]
    fn write_to_vram_address_0x3F1F_writes_to_end_of_palette_ram() {
        let mut vram = create_test_vram();
        vram.write(0x3F1F, 0xFE);
        assert_eq!(0xFE, vram.palette_memory[0x1F]);
    }


    #[test]
    fn read_from_vram_address_0x3F1F_reads_from_end_of_palette_ram() {
        let mut vram = create_test_vram();
        vram.palette_memory[0x1F] = 0x15;
        assert_eq!(0x15, vram.read(0x3F1F));
    }

    #[test]
    fn write_to_vram_address_0x3F10_is_mirrored_to_0x3F00() {
        let mut vram = create_test_vram();
        vram.write(0x3F10, 0xFE);
        assert_eq!(0xFE, vram.read(0x3F00));
    }

    #[test]
    fn read_from_vram_address_0x3F10_is_mirrored_to_0x3F00() {
        let mut vram = create_test_vram();
        vram.write(0x3F00, 0xFA);
        assert_eq!(0xFA, vram.read(0x3F10));
    }

    #[test]
    fn write_to_vram_address_0x3F14_is_mirrored_to_0x3F04() {
        let mut vram = create_test_vram();
        vram.write(0x3F14, 0xFE);
        assert_eq!(0xFE, vram.read(0x3F04));
    }

    #[test]
    fn read_from_vram_address_0x3F14_is_mirrored_to_0x3F04() {
        let mut vram = create_test_vram();
        vram.write(0x3F04, 0xFA);
        assert_eq!(0xFA, vram.read(0x3F14));
    }


    #[test]
    fn write_to_vram_address_0x3F18_is_mirrored_to_0x3F08() {
        let mut vram = create_test_vram();
        vram.write(0x3F18, 0xFE);
        assert_eq!(0xFE, vram.read(0x3F08));
    }

    #[test]
    fn read_from_vram_address_0x3F18_is_mirrored_to_0x3F08() {
        let mut vram = create_test_vram();
        vram.write(0x3F08, 0xFA);
        assert_eq!(0xFA, vram.read(0x3F18));
    }

    #[test]
    fn write_to_vram_address_0x3F1C_is_mirrored_to_0x3F0C() {
        let mut vram = create_test_vram();
        vram.write(0x3F1C, 0xFE);
        assert_eq!(0xFE, vram.read(0x3F0C));
    }

    #[test]
    fn read_from_vram_address_0x3F1C_is_mirrored_to_0x3F0C() {
        let mut vram = create_test_vram();
        vram.write(0x3F0C, 0xFA);
        assert_eq!(0xFA, vram.read(0x3F1C));
    }


    #[test]
    fn write_to_vram_address_0x3F20_is_mirrored_to_0x3F00() {
        let mut vram = create_test_vram();
        vram.write(0x3F20, 0xFE);
        assert_eq!(0xFE, vram.read(0x3F00));
    }

    #[test]
    fn read_from_vram_address_0x3F20_is_mirrored_to_0x3F00() {
        let mut vram = create_test_vram();
        vram.write(0x3F00, 0xFA);
        assert_eq!(0xFA, vram.read(0x3F20));
    }

    #[test]
    fn write_to_vram_address_0x3FFF_is_mirrored_to_0x3F1F() {
        let mut vram = create_test_vram();
        vram.write(0x3F1F, 0xFE);
        assert_eq!(0xFE, vram.read(0x3FFF));
    }

    #[test]
    fn read_from_vram_address_0x3FFF_is_mirrored_to_0x3F1F() {
        let mut vram = create_test_vram();
        vram.write(0x3FFF, 0xFA);
        assert_eq!(0xFA, vram.read(0x3F1F));
    }

    #[test]
    fn write_to_vram_address_0x3F45_is_mirrored_to_0x3F05() {
        let mut vram = create_test_vram();
        vram.write(0x3F45, 0xFE);
        assert_eq!(0xFE, vram.read(0x3F05));
    }

    #[test]
    fn read_from_vram_address_0x3F45_is_mirrored_to_0x3F05() {
        let mut vram = create_test_vram();
        vram.write(0x3F05, 0xFA);
        assert_eq!(0xFA, vram.read(0x3F45));
    }
}
