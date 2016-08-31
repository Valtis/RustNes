mod dmc;
mod frame_counter;

use memory::Memory;
use std::fmt;
use self::dmc::Dmc;

pub struct Apu {

    dmc: Dmc,
}


impl Memory for Apu {
    fn read(&mut self, address: u16) -> u8 {
     //   panic!("Read from APU register {:#x} not implemented", address);
     0
    }

    fn write(&mut self, address: u16, value: u8) {
        if address >= 0x4010 && address <= 0x4013 {
            self.dmc.write(address, value);
        } else {
         //   panic!("Write to APU register {:#x} not implemented (value: {})", address, value);
        }
    }

}

impl fmt::Debug for Apu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Memory contents not included>")
    }
}



impl Apu {
    pub fn new() -> Apu {
        let dmc = Dmc::new();
        Apu {
            dmc: dmc
        }
    }
}
