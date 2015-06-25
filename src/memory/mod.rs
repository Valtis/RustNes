use rom::Rom;


pub struct Memory {
    rom: Rom,
    memory: [u8; 0xffff + 1],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            rom: Rom::new(),
            memory: [0; 0xffff + 1],
        }
    }

    pub fn set_rom(&mut self, rom: Rom) {
        self.rom = rom;
        // TODO - HANDLE ACTUAL MAPPERS. For now hardcoded for NROM
        // write 0x800 into 0xfffc - this is the initialization vector and is the location of first
        // instruction
        /*self.memory[0xFFFC] = 0x00;
        self.memory[0xFFFD] = 0x80;*/

        // if size is 16kb, map rom into 0x8000 - 0xbfff; mirror data to 0xc000 - 0xffff
        // otherwise map first 16kb into 0x8000 - 0xbfff and second 16k into 0xc000 - 0xffff

        if self.rom.prg_rom_data.len() == 0x4000 {
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
        }
    }

    pub fn read(&self, location: u16) -> u8 {
        self.memory[location as usize]
    }

    pub fn write(&mut self, location: u16, value: u8) {
        self.memory[location as usize] = value;
    }
}
