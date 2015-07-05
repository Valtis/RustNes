use memory::*;
use rom::*;
use ram::*;

pub struct MemoryBus {
    rom: Rom,
    ram: Box<Memory>,
}

impl Memory for MemoryBus {
    fn read(&self, address: u16) -> u8 {
        panic!("Unimplemented");
    }

    fn write(&mut self, address: u16, value: u8) {
        panic!("Unimplemented");
    }

}

impl MemoryBus {
    pub fn new(rom: Rom) -> MemoryBus  {
        MemoryBus {
            rom: rom,
            ram: Box::new(Ram::new()) as Box<Memory>,
        }
    }

    fn mapper_0(&mut self) {
        // if size is 16kb, map rom into 0x8000 - 0xbfff; mirror data to 0xc000 - 0xffff
        // otherwise map first 16kb into 0x8000 - 0xbfff and second 16k into 0xc000 - 0xffff
/*        if self.rom.prg_rom_data.len() == 0x4000 {
            for i in 0..0x4000 {
                self.memory[0x8000 + i] = self.rom.prg_rom_data[i];
                self.memory[0xc000 + i] = self.rom.prg_rom_data[i];
            }
        } else {
            for i in 0..0x4000 {
                self.memory[0x8000 + i] = self.rom.prg_rom_data[i];
            }
            let mut pos = 0;
            for i in 0x4000..self.rom.prg_rom_data.len() {

                let f =  self.rom.prg_rom_data[i];
                self.memory[0xc000 + pos] = f;
                pos += 1;
            }
        }*/
    }
}
