use memory::*;
use ram::*;
use ppu::*;
use apu::*;
use controller::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct MemoryBus<'a> {
    rom: Rc<RefCell<Box<Memory>>>,
    ram: Box<Memory>,
    ppu: Rc<RefCell<Ppu<'a>>>,
    apu: Rc<RefCell<Apu<'a>>>,
    controllers: Vec<Rc<RefCell<Controller>>>,
}


impl<'a> Memory for MemoryBus<'a> {
    fn read(&mut self, address: u16) -> u8 {
        if address < 0x2000 {
            self.ram.read(address)
        } else if (address >= 0x2000 && address <= 0x3FFF) || address == 0x4014 {
            self.ppu.borrow_mut().read(address)
        } else if address == 0x4016 {
            self.controllers[0].borrow_mut().read(address)
        } else if address == 0x04017 {
            self.controllers[1].borrow_mut().read(address)
        } else if (address >= 0x4000 && address <= 0x4015) || address == 0x4017 {
            self.apu.borrow_mut().read(address)
        } else if address >= 0x4020 {
            self.rom.borrow_mut().read(address)
        } else {
            0
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            self.ram.write(address, value);
        } else if address >= 0x2000 && address <= 0x3FFF {
            self.ppu.borrow_mut().write(address, value);
        } else if address == 0x4014 {
            let start = (value as u16) << 8;
            let mut data = vec![];

            for i in start..(start + 0x100) {
                data.push(self.read(i));
            }

            self.ppu.borrow_mut().oam_dma_write(data);
        } else if address == 0x4016 {
            self.controllers[0].borrow_mut().write(address, value);
            self.controllers[1].borrow_mut().write(address, value);
        } else if (address >= 0x4000 && address <= 0x4015) || address == 0x4017 {
            self.apu.borrow_mut().write(address, value);
        } else if address >= 0x4020 {
            self.rom.borrow_mut().write(address, value);
        }
    }

}

impl<'a> MemoryBus<'a> {
    pub fn new(rom: Rc<RefCell<Box<Memory>>>,
               ppu: Rc<RefCell<Ppu<'a>>>,
               apu: Rc<RefCell<Apu<'a>>>,
               controllers: Vec<Rc<RefCell<Controller>>>) -> MemoryBus<'a>  {
        MemoryBus {
            rom: rom,
            ram: Box::new(Ram::new()) as Box<Memory>,
            ppu: ppu,
            apu: apu,
            controllers: controllers,
        }
    }
}


#[cfg(test)]
mod tests {

    extern crate sdl2;

    use super::*;
    use memory::*;
    use ppu::*;
    use rom::*;
    use ppu::renderer::*;
    use apu::{Apu, Audio};
    use std::cell::RefCell;
    use std::rc::Rc;
    use self::sdl2::audio::{AudioQueue};


    // 64 kilobytes of memory, no mapped addresses
    struct MockRenderer;

    impl MockRenderer {
        fn new() -> MockRenderer {
            MockRenderer
        }
    }

    impl Renderer for MockRenderer {
        fn render(&mut self, pixels: &Vec<Pixel>) {

        }
    }
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

    // few helpers
    impl<'a> MemoryBus<'a> {
        fn assert_value_present_in_ram_only(&mut self, address: u16, value: u8) {
            assert_eq!(value, self.ram.read(address));
            assert!(self.rom.borrow_mut().read(address) != value);
        }

        fn assert_value_present_in_rom_only(&mut self, address: u16, value: u8) {
            assert_eq!(value, self.rom.borrow_mut().read(address));
            assert!(self.ram.read(address) != value);
        }
    }

    fn create_test_memory_bus<'a>() -> MemoryBus<'a> {

        let rom = Rc::new(RefCell::new(Box::new(MockMemory::new()) as Box<Memory>));
        MemoryBus {
            rom: rom.clone(),
            ram: Box::new(MockMemory::new()),
            ppu: Rc::new(RefCell::new(Ppu::new(Box::new(MockRenderer::new()), TvSystem::NTSC, Mirroring::VerticalMirroring, rom.clone()))),
            controllers: vec![],
            apu: Rc::new(RefCell::new(Apu::new(Box::new(MockAudio::new())))),
        }
    }

    struct MockAudio {
    }

    impl MockAudio {
        fn new() -> MockAudio {
            MockAudio {

            }
        }
    }

    impl Audio<f32> for MockAudio {
        fn queue(&mut self, slice: &[f32]) {

        }
    }

    #[test]
    fn write_under_0x2000_is_redirected_to_ram() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.write(0x1800, 0x4B);
        mem_bus.assert_value_present_in_ram_only(0x1800, 0x4B);
    }

    #[test]
    fn write_to_0x0000_is_redirected_to_ram() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.write(0x0000, 0x4B);
        mem_bus.assert_value_present_in_ram_only(0x0000, 0x4B);
    }

    #[test]
    fn write_to_0x1FFF_is_redirected_to_ram() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.write(0x1FFF, 0x4B);
        mem_bus.assert_value_present_in_ram_only(0x1FFF, 0x4B);
    }

    #[test]
    fn read_under_0x2000_is_read_from_ram() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.ram.write(0x456, 0x4B);
        assert_eq!(0x4B, mem_bus.read(0x456));
    }

    #[test]
    fn read_at_0x0000_is_read_from_ram() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.ram.write(0x0000, 0x4B);
        assert_eq!(0x4B, mem_bus.read(0x0000));
    }

    #[test]
    fn read_at_0x1FFF_is_read_from_ram() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.ram.write(0x1FFF, 0x4B);
        assert_eq!(0x4B, mem_bus.read(0x1FFF));
    }

    #[test]
    fn write_above_0x4020_is_redirected_to_rom() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.write(0x7090, 0x4B);
        mem_bus.assert_value_present_in_rom_only(0x7090, 0x4B);
    }

    #[test]
    fn write_to_0x4020_is_redirected_to_rom() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.write(0x4020, 0x4B);
        mem_bus.assert_value_present_in_rom_only(0x4020, 0x4B);
    }

    #[test]
    fn write_to_0xFFFF_is_redirected_to_rom() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.write(0xFFFF, 0x4B);
        mem_bus.assert_value_present_in_rom_only(0xFFFF, 0x4B);
    }

    #[test]
    fn read_above_0x4020_is_read_from_rom() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.rom.borrow_mut().write(0xEFFF, 0x4B);
        assert_eq!(0x4B, mem_bus.read(0xEFFF));
    }

    #[test]
    fn read_at_0x4020_is_read_from_rom() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.rom.borrow_mut().write(0x4020, 0x4B);
        assert_eq!(0x4B, mem_bus.read(0x4020));
    }

    #[test]
    fn read_at_0xFFFF_is_read_from_rom() {
        let mut mem_bus = create_test_memory_bus();
        mem_bus.rom.borrow_mut().write(0xFFFF, 0x4B);
        assert_eq!(0x4B, mem_bus.read(0xFFFF));
    }
}
