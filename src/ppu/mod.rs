mod vram;
mod tv_system_values;

use memory::Memory;
use rom::TvSystem;
use self::vram::Vram;
use self::tv_system_values::TvSystemValues;

use std::fmt;

pub struct Ppu {
    object_attribute_memory: Vec<u8>,
    vram: Box<Memory>,
    registers: Registers,
    address_latch: bool,
    vram_address: u16,
    vram_read_buffer: u8,
    tv_system: TvSystemValues,
    current_scanline: u16,
    pos_at_scanline: u16,
    nmi_occured: bool,
}



impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:?}", self.registers));
        write!(f, "<Memory contents not included>")
    }
}

impl Memory for Ppu {
    fn read(&mut self, cpu_address: u16) -> u8 {
        match cpu_address & 0x0007 {
            0 => panic!("Attempting to read from write-only ppu control register (address 0x{:04X})", cpu_address),
            1 => panic!("Attempting to read from write-only ppu mask register (address 0x{:04X})", cpu_address),
            2 => self.status_register_read(),
            3 => panic!("Attempting to read from write-only ppu oam address register (address 0x{:04X})", cpu_address),
            4 => self.oam_data_register_read(),
            5 => panic!("Attempting to read from write-only ppu scroll register (address 0x{:04X})", cpu_address),
            6 => panic!("Attempting to read from write-only ppu address register (address 0x{:04X})", cpu_address),
            7 => self.ppu_data_register_read(),
            _ => panic!("Something went horribly wrong in modulus calculation in ppu.read (address: {})", cpu_address),
        }
    }

    fn write(&mut self, cpu_address: u16, value: u8) {
        match cpu_address & 0x0007 {
            0 => self.control_register_write(value),
            1 => self.registers.mask = value,
            2 => panic!("Attempting to write to read-only ppu status register (address 0x{:04X})", cpu_address),
            3 => self.registers.oam_address = value,
            4 => self.oam_data_register_write(value),
            5 => self.scroll_register_write(value),
            6 => self.ppu_address_register_write(value),
            7 => self.ppu_data_register_write(value),
            _ => panic!("Something went horribly wrong in modulus calculation in ppu.write (address: {})", cpu_address),
        }
    }
}

impl Ppu {
    pub fn new(tv_system: &TvSystem) -> Ppu {
        Ppu {
            object_attribute_memory: vec![0;256],
            vram: Box::new(Vram::new()),
            registers: Registers::new(),
            address_latch: false,
            vram_address: 0,
            vram_read_buffer: 0,
            tv_system: TvSystemValues::new(&tv_system),
            current_scanline: 0,
            pos_at_scanline: 0,
            nmi_occured: false,
        }
    }

    fn generate_nmi_if_flags_set(&mut self) {
        if self.registers.control & 0x80 == 0x80 && self.registers.status & 0x80 == 0x80 {
            self.nmi_occured = true;
        }
    }

    pub fn nmi_occured(&mut self) -> bool {
        let occured = self.nmi_occured;
        self.nmi_occured = false;
        occured
    }

    fn increment_vram(&mut self) {
        if self.registers.control & 0x04 == 0 {
            self.vram_address += 1;
        }  else {
            self.vram_address += 32;
        }
    }

    fn control_register_write(&mut self, value: u8) {
        self.registers.control = value;
        self.generate_nmi_if_flags_set();
    }

    fn status_register_read(&mut self) -> u8 {
        self.address_latch = false;
        let val = self.registers.status;
        self.registers.status = self.registers.status & 0x7F;
        val
    }

    fn oam_data_register_read(&mut self) -> u8 {
        let address = self.registers.oam_address as usize;
        self.object_attribute_memory[address]
    }

    fn oam_data_register_write(&mut self, value: u8) {
        let address = self.registers.oam_address as usize;
        self.registers.oam_address += 1;
        self.object_attribute_memory[address] = value;
    }

    fn scroll_register_write(&mut self, value: u8) {
        self.address_latch = !self.address_latch;
        self.registers.scroll = value;
    }

    fn ppu_address_register_write(&mut self, value: u8) {
        if self.address_latch {
            self.vram_address = self.vram_address & 0xFF00 | value as u16;
        } else {
            self.vram_address = self.vram_address & 0x00FF | ((value as u16) << 8);
        }
        self.address_latch = !self.address_latch;
    }

    fn ppu_data_register_read(&mut self) -> u8 {
        let address = self.vram_address;
        self.increment_vram();

        if address <= 0x3EFF {
            // read is buffered; return value in buffer and update buffer to value at address
            let buffer = self.vram_read_buffer;
            self.vram_read_buffer = self.vram.read(address);
            buffer
        } else {
            // read is not buffered; update buffer to value at address - 0x1000
            let value = self.vram.read(address);
            self.vram_read_buffer = self.vram.read(address - 0x1000);
            value
        }
    }

    fn ppu_data_register_write(&mut self, value: u8) {
        let address = self.vram_address;
        self.increment_vram();
        self.vram.write(address, value);
    }

    pub fn oam_dma_write(&mut self, data: Vec<u8>) {
        if data.len() != 256 {
            panic!("Invalid oam data write length: 256 expected but was {}", data.len());
        }
        self.object_attribute_memory = data;
    }

    // how many cycles will be executed this cpu cycle
    // also updates counters
    fn get_cycle_count(&mut self) -> u8 {
        if self.tv_system.ppu_extra_cycle_every_cpu_cycle != 0 {
            self.tv_system.extra_cycle_counter += 1;
            if self.tv_system.extra_cycle_counter == self.tv_system.ppu_extra_cycle_every_cpu_cycle {
                self.tv_system.extra_cycle_counter = 0;
                self.tv_system.ppu_cycles_per_cpu_cycle + 1
            } else {
                self.tv_system.ppu_cycles_per_cpu_cycle
            }
        } else {
            self.tv_system.ppu_cycles_per_cpu_cycle
        }
    }

    pub fn execute_cycles(&mut self) {
        let cycles = self.get_cycle_count();
        for _ in 0..cycles {
            self.execute_cycle();
        }
    }

    fn execute_cycle(&mut self) {
//        println!("Scanline: {}, pos at scanline: {}", self.current_scanline, self.pos_at_scanline);
        let rendered_scanlines = 240;
        if self.current_scanline < self.tv_system.vblank_frames {
            self.do_vblank();
        } else if self.current_scanline == self.tv_system.vblank_frames {
            self.do_pre_render_line();
        } else if self.current_scanline > self.tv_system.vblank_frames
            && self.current_scanline <= self.tv_system.vblank_frames + rendered_scanlines {
            self.do_render();
        } else if self.current_scanline > self.tv_system.vblank_frames + rendered_scanlines
            && self.current_scanline <= self.tv_system.vblank_frames + rendered_scanlines + self.tv_system.post_render_scanlines {
            // post render line - do nothing.
            self.dummy_scanline()
        } else {
            self.current_scanline = 0;
            self.execute_cycle();
        }
    }

    fn do_vblank(&mut self) {
        // VBLANK - do not access memory or render.
        // However, set vblank flag on second tick of first scanline and raise NMI if nmi flag is set
        if self.current_scanline == 0 && self.pos_at_scanline == 1 {
            self.registers.status = self.registers.status | 0x80;
            self.generate_nmi_if_flags_set();
        }
        self.dummy_scanline();
    }

    fn do_pre_render_line(&mut self) {
        if self.pos_at_scanline == 1 {
            self.registers.status = self.registers.status & 0x7F;
        }
        self.dummy_scanline();
    }

    fn dummy_scanline(&mut self) {
        self.pos_at_scanline += 1;
        if self.pos_at_scanline == 340 {
            self.pos_at_scanline = 0;
            self.current_scanline += 1;
        }
    }

    fn do_render(&mut self) {
        self.dummy_scanline();
    }

}

#[derive(Debug)]
struct Registers {
    control: u8,
    mask: u8,
    status: u8,
    oam_address: u8,
    oam_dma: u8,
    scroll: u8,
}

impl Registers {
    fn new() -> Registers {
        Registers {
            control: 0,
            mask: 0,
            status: 0,
            oam_address: 0,
            oam_dma: 0,
            scroll: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;
    use rom::TvSystem;

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

    fn create_test_ppu() -> Ppu {
        let mut ppu = Ppu::new(&TvSystem::NTSC);
        // replace vram with mock
        ppu.vram = Box::new(MockMemory::new());
        ppu
    }

    #[test]
    fn write_to_0x2000_changes_control_register_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2000, 0x13);
        assert_eq!(0x13, ppu.registers.control);
    }

    #[test]
    fn write_to_0x2000_generates_nmi_if_nmi_bit_will_be_set_and_vblank_bit_is_set() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0x80;
        ppu.nmi_occured = false;
        ppu.write(0x2000, 0x83);
        assert_eq!(true, ppu.nmi_occured);
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
    fn write_to_0x2004_changes_object_attribute_memory_at_oam_address() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_address = 0xFE;
        ppu.write(0x2004, 0x13);
        assert_eq!(0x13, ppu.object_attribute_memory[0xFE]);
    }

    #[test]
    fn write_to_0x2004_increments_oam_address_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_address = 0x21;
        ppu.write(0x2004, 0x34);
        assert_eq!(0x22, ppu.registers.oam_address);
    }

    #[test]
    fn read_from_0x2004_returns_oam_value_at_address() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_address = 0x56;
        ppu.object_attribute_memory[0x56] = 0x64;
        assert_eq!(0x64, ppu.read(0x2004));
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
    fn write_to_0x2006_writes_high_byte_if_latch_is_unset() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.vram_address = 0x1234;
        ppu.write(0x2006, 0x13);
        assert_eq!(0x1334, ppu.vram_address);
    }

    #[test]
    fn write_to_0x2006_writes_low_byte_if_latch_is_set() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.vram_address = 0x1234;
        ppu.write(0x2006, 0x6F);
        assert_eq!(0x126F, ppu.vram_address);
    }

    #[test]
    fn write_to_0x2007_writes_to_memory_at_vram_address() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3435;
        ppu.write(0x2007, 0x13);
        assert_eq!(0x13, ppu.vram.read(0x3435));
    }

    #[test]
    fn read_from_0x2007_returns_vram_read_buffer_value_when_reading_below_0x3F00() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3EFF;
        ppu.vram_read_buffer = 0xA4;
        ppu.vram.write(0x3EFF, 0x23);
        assert_eq!(0xA4, ppu.read(0x2007));
    }

    #[test]
    fn read_from_0x2007_updates_read_buffer_to_value_at_current_address_when_reading_below_0x3F00() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3EFF;
        ppu.vram_read_buffer = 0xA4;
        ppu.vram.write(0x3EFF, 0xB9);
        ppu.read(0x2007);
        assert_eq!(0xB9, ppu.vram_read_buffer);
    }

    #[test]
    fn read_from_0x2007_returns_data_straight_from_vram_skipping_read_buffer_when_reading_at_or_above_0x3F00() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3F00;
        ppu.vram_read_buffer = 0xE1;
        ppu.vram.write(0x3F00, 0xBE);
        assert_eq!(0xBE, ppu.read(0x2007));
    }

    #[test]
    fn read_from_0x2007_sets_read_buffer_to_nametable_value_at_0x1000_before_address() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3F00;
        ppu.vram_read_buffer = 0x14;
        ppu.vram.write(0x3F00, 0x200);
        ppu.vram.write(0x2F00, 0xB1);
        ppu.read(0x2007);
        assert_eq!(0xB1, ppu.vram_read_buffer);
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
        ppu.object_attribute_memory[0xA9] = 0xF2;
        ppu.registers.oam_address = 0xA9;
        assert_eq!(0xF2, ppu.read(0x3014));
    }

    #[test]
    // write to data register -> write to vram at vram_address
    fn write_to_0x3FFF_writes_to_ppu_data_register() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x1234;
        ppu.write(0x3FFF, 0x13);
        assert_eq!(0x13, ppu.vram.read(0x1234));
    }

    #[test]
    fn read_from_0x3FFF_reads_from_ppu_data_register() {
        let mut ppu = create_test_ppu();
        ppu.vram_read_buffer = 0xF2;
        ppu.vram_address = 0x1234;
        assert_eq!(0xF2, ppu.read(0x3FFF));
    }

    #[test]
    #[should_panic]
    fn oam_dma_write_panics_if_data_length_is_not_256_bytes() {
        let data:Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut ppu = create_test_ppu();
        ppu.oam_dma_write(data);
    }

    #[test]
    fn oam_dma_write_replaces_oam() {
        let mut ppu = create_test_ppu();
        let mut data = vec![];
        for i in 0..256 {
            data.push(((i as u16) % 256) as u8);
        }
        ppu.oam_dma_write(data.clone());
        assert_eq!(data, ppu.object_attribute_memory);
    }

    #[test]
    fn get_cycle_count_returns_same_value_every_time_if_no_extra_cycles_are_needed() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 4;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 0;
        for _ in 0..100 {
            assert_eq!(4, ppu.get_cycle_count());
        }
    }

    #[test]
    fn get_cycle_count_does_not_modify_extra_cycle_counter_if_no_extra_cycles_are_needed() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 4;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 0;
        for _ in 0..100 {
            ppu.get_cycle_count();
            assert_eq!(0, ppu.tv_system.extra_cycle_counter);
        }
    }

    #[test]
    fn get_cycle_count_returns_extra_cycle_every_n_frames_if_cycle_is_required() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 3;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 5;
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(4, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
    }

    #[test]
    fn get_cycle_count_modifies_extra_cycle_counter_if_extra_cycle_is_needed() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 4;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 7;
        for i in 1..100 {
            ppu.get_cycle_count();
            assert_eq!(i % 7, ppu.tv_system.extra_cycle_counter);
        }
    }

    #[test]
    fn ppu_sets_vblank_bit_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_set() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x80;
        ppu.registers.control = 0;
        ppu.execute_cycle();
        assert_eq!(0x80, ppu.registers.status & 0x80);
    }

    #[test]
    fn ppu_sets_vblank_bit_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_cleared() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0;
        ppu.registers.control = 0;
        ppu.execute_cycle();
        assert_eq!(0x80, ppu.registers.status & 0x80);
    }

    #[test]
    fn ppu_generates_nmi_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_set() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x00;
        ppu.registers.control = 0x80;
        ppu.execute_cycle();
        assert_eq!(true, ppu.nmi_occured);
    }

    #[test]
    fn ppu_does_not_generate_nmi_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_cleared() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x00;
        ppu.registers.control = 0x00;
        ppu.execute_cycle();
        assert_eq!(false, ppu.nmi_occured);
    }

    #[test]
    fn ppu_clears_vblank_bit_on_pre_render_scanline_second_pixel() {
        let mut ppu = create_test_ppu();

        ppu.tv_system.vblank_frames = 50;
        ppu.current_scanline = 50;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x80;
        ppu.registers.control = 0;

        ppu.execute_cycle();

        assert_eq!(0x00, ppu.registers.status);
    }

    #[test]
    fn nmi_occured_returns_true_if_nmi_has_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = true;
        assert_eq!(true, ppu.nmi_occured());
    }

    #[test]
    fn nmi_occured_returns_false_if_nmi_has_not_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = false;
        assert_eq!(false, ppu.nmi_occured());
    }
    #[test]
    fn nmi_occured_clears_nmi_status_if_nmi_has_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = true;
        ppu.nmi_occured();
        assert_eq!(false, ppu.nmi_occured);
    }
    #[test]
    fn nmi_does_nothing_to_nmi_status_nmi_has_not_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = false;
        ppu.nmi_occured();
        assert_eq!(false, ppu.nmi_occured);
    }


}
