
use memory::Memory;

use std::fmt;
pub struct Ppu {
    object_attribute_memory: Vec<u8>,
    memory: Vec<u8>,
    registers: Registers,
    address_latch: bool,
    vram_address: u16,
}



impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.registers);
        write!(f, "<Memory contents not included>")
    }
}

impl Memory for Ppu {
    fn read(&mut self, address: u16) -> u8 {
        match address & 0x0007 {
            0 => panic!("Attempting to read from write-only ppu control register (address 0x{:04X})", address),
            1 => panic!("Attempting to read from write-only ppu mask register (address 0x{:04X})", address),
            2 => self.status_register_read(),
            3 => panic!("Attempting to read from write-only ppu oam address register (address 0x{:04X})", address),
            4 => self.registers.oam_data,
            5 => panic!("Attempting to read from write-only ppu scroll register (address 0x{:04X})", address),
            6 => panic!("Attempting to read from write-only ppu address register (address 0x{:04X})", address),
            7 => self.ppu_data_register_read(),
            _ => panic!("Something went horribly wrong in modulus calculation in ppu.read (address: {})", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address & 0x0007 {
            0 => self.registers.control = value,
            1 => self.registers.mask = value,
            2 => panic!("Attempting to write to read-only ppu status register (address 0x{:04X})", address),
            3 => self.registers.oam_address = value,
            4 => self.oam_data_register_write(value),
            5 => self.scroll_register_write(value),
            6 => self.ppu_address_register_write(value),
            7 => self.ppu_data_register_write(value),
            _ => panic!("Something went horribly wrong in modulus calculation in ppu.write (address: {})", address),
        }
    }
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            object_attribute_memory: vec![0;256],
            memory: vec![0;0x800], // 2kb of internam memory
            registers: Registers::new(),
            address_latch: false,
            vram_address: 0,
        }
    }

    fn increment_vram(&mut self) {
        if self.registers.control & 0x04 == 0 {
            self.vram_address += 1;
        }  else {
            self.vram_address += 32;
        }
    }

    fn status_register_read(&mut self) -> u8 {
        self.address_latch = false;
        let val = self.registers.status;
        self.registers.status = self.registers.status & 0x7F;
        val
    }

    fn oam_data_register_write(&mut self, value: u8) {
        self.registers.oam_data = value;
        self.registers.oam_address += 1;
    }

    fn scroll_register_write(&mut self, value: u8) {
        self.address_latch = !self.address_latch;
        self.registers.scroll = value;
    }

    fn ppu_address_register_write(&mut self, value: u8) {
        self.address_latch = !self.address_latch;
        self.registers.ppu_address = value;
    }

    fn ppu_data_register_read(&mut self) -> u8 {
        self.increment_vram();
        self.registers.ppu_data
    }

    fn ppu_data_register_write(&mut self, value: u8) {
        self.registers.ppu_data = value;
        self.increment_vram();
    }


}

#[derive(Debug)]
struct Registers {
    control: u8,
    mask: u8,
    status: u8,
    oam_address: u8,
    oam_data: u8,
    oam_dma: u8,
    scroll: u8,
    ppu_address: u8,
    ppu_data: u8,
}

impl Registers {
    fn new() -> Registers {
        Registers {
            control: 0,
            mask: 0,
            status: 0,
            oam_address: 0,
            oam_data: 0,
            oam_dma: 0,
            scroll: 0,
            ppu_address: 0,
            ppu_data: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;

    fn create_test_ppu() -> Ppu {
        Ppu::new()
    }

    #[test]
    fn write_to_0x2000_changes_control_register_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2000, 0x13);
        assert_eq!(0x13, ppu.registers.control);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2000_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2000);
    }

    #[test]
    fn write_to_0x2001_changes_mask_register_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2001, 0x13);
        assert_eq!(0x13, ppu.registers.mask);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2001_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2001);
    }

    #[test]
    #[should_panic]
    fn write_to_0x2002_panics() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2002, 0xD4);
    }

    #[test]
    fn read_from_0x2002_returns_status_register_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0xD5;
        assert_eq!(0xD5, ppu.read(0x2002));
    }

    #[test]
    fn read_from_0x2002_clears_the_bit_7() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0xD5;
        ppu.read(0x2002);
        assert_eq!(0x55, ppu.registers.status);
    }

    #[test]
    fn read_from_0x2002_clears_the_address_latch() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.read(0x2002);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    fn read_from_0x2002_does_not_flip_the_address_latch_if_latch_is_clear() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.read(0x2002);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2003_changes_oam_address_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2003, 0x13);
        assert_eq!(0x13, ppu.registers.oam_address);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2003_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2003);
    }

    #[test]
    fn write_to_0x2004_changes_oam_data_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2004, 0x13);
        assert_eq!(0x13, ppu.registers.oam_data);
    }

    #[test]
    fn write_to_0x2004_increments_oam_address_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_address = 0x21;
        ppu.write(0x2004, 0x34);
        assert_eq!(0x22, ppu.registers.oam_address);
    }

    #[test]
    fn read_from_0x2004_returns_oam_data_register_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_data = 0x42;
        assert_eq!(0x42, ppu.read(0x2004));
    }

    #[test]
    fn write_to_0x2005_changes_scroll_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2005, 0x13);
        assert_eq!(0x13, ppu.registers.scroll);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2005_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2005);
    }

    #[test]
    fn write_to_0x2005_sets_the_address_latch_if_it_was_unset_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.write(0x2005, 0x13);
        assert_eq!(true, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2005_unsets_the_address_latch_if_it_was_set_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.write(0x2005, 0x13);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2006_changes_ppu_address_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2006, 0x13);
        assert_eq!(0x13, ppu.registers.ppu_address);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2006_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2006);
    }

    #[test]
    fn write_to_0x2006_sets_the_address_latch_if_it_was_unset_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.write(0x2006, 0x13);
        assert_eq!(true, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2006_unsets_the_address_latch_if_it_was_set_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.write(0x2006, 0x13);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2007_changes_ppu_data_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2007, 0x13);
        assert_eq!(0x13, ppu.registers.ppu_data);
    }

    #[test]
    fn read_from_0x2007_returns_ppu_data_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.ppu_data =  0x13;
        assert_eq!(0x13, ppu.read(0x2007));
    }

    #[test]
    fn read_from_0x2007_increments_vram_address_by_one_if_control_register_bit_2_is_0() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x00;
        ppu.vram_address = 0x2353;
        ppu.read(0x2007);
        assert_eq!(0x2354, ppu.vram_address);
    }

    #[test]
    fn read_from_0x2007_increments_vram_address_by_32_if_control_register_bit_2_is_1() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x04;
        ppu.vram_address = 0x2353;
        ppu.read(0x2007);
        assert_eq!(0x2353 + 32, ppu.vram_address);
    }

    #[test]
    fn write_to_0x2007_increments_vram_address_by_one_if_control_register_bit_2_is_0() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x00;
        ppu.vram_address = 0x2353;
        ppu.write(0x2007, 0x23);
        assert_eq!(0x2354, ppu.vram_address);
    }

    #[test]
    fn write_to_0x2007_increments_vram_address_by_32_if_control_register_bit_2_is_1() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x04;
        ppu.vram_address = 0x2353;
        ppu.write(0x2007, 0x23);
        assert_eq!(0x2353 + 32, ppu.vram_address);
    }

    #[test]
    fn write_to_address_0x2008_is_mirrored_to_control_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2008, 0x13);
        assert_eq!(0x13, ppu.registers.control);
    }

    #[test]
    fn read_from_address_0x200A_is_mirrored_to_status_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0xD5;
        assert_eq!(0xD5, ppu.read(0x200A));
    }

    #[test]
    fn write_to_address_between_0x2008_and_0x3FFF_is_mirrored_correctly() {
        let mut ppu = create_test_ppu();
        ppu.write(0x3013, 0x13);
        assert_eq!(0x13, ppu.registers.oam_address);
    }

    #[test]
    fn read_from_address_between_0x2008_and_0x3FFF_is_mirrored_correctly() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_data = 0xF2;
        assert_eq!(0xF2, ppu.read(0x3014));
    }

    #[test]
    fn write_to_0x3FFF_writes_to_ppu_data_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x3FFF, 0x13);
        assert_eq!(0x13, ppu.registers.ppu_data);
    }

    #[test]
    fn read_from_address_0x3FFF_returns_ppu_data_register_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.ppu_data = 0xF2;
        assert_eq!(0xF2, ppu.read(0x3FFF));
    }
}
