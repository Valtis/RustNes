
use std::fmt;
pub struct Ppu {
    object_attribute_memory: Vec<u8>,
    memory: Vec<u8>,
    registers: Registers,
}



impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.registers);
        write!(f, "<Memory contents not included>")
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
    address: u8,
    data: u8,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            object_attribute_memory: vec![0;256],
            memory: vec![0;0x800], // 2kb of internam memory
            registers: Registers::new(),
        }
    }
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
            address: 0,
            data: 0,
        }
    }
}

#[cfg(test)]
mod tests {


}
