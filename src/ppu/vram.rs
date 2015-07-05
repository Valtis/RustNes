use memory::Memory;
use rom::ArrangementMirroring;

pub struct Vram {
    memory: Vec<u8>,
    mirroring: ArrangementMirroring,
}

impl Vram {
    pub fn new() -> Vram {
        Vram {
            memory: vec![0;0x0800],
            mirroring: ArrangementMirroring::VArrangementHMirroring, // temporary hardcoding
        }
    }
}


impl Memory for Vram {
    fn read(&mut self, address: u16) -> u8 {
        panic!("Not implemented");
    }

    fn write(&mut self, address: u16, value: u8) {
        panic!("Not implemented");
    }
}
