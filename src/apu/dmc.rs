use memory::Memory;


static NTSC_RATE: [u16;16] = [ 428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54 ];
static PAL_RATE: [u16;16] = [ 398, 354, 316, 298, 276, 236, 210, 198, 176, 148, 132, 118, 98, 78, 66, 50 ];

pub struct Dmc {
    // IL--RRRR
    // I: IRQ enabled flag. If clear, the interrupt flag is cleared.
    // L: Loop flag
    // RRRR: Rate index
    // Rate   $0   $1   $2   $3   $4   $5   $6   $7   $8   $9   $A   $B   $C   $D   $E   $F
    //  ------------------------------------------------------------------------------
    // NTSC  428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54
    // PAL   398, 354, 316, 298, 276, 236, 210, 198, 176, 148, 132, 118,  98,  78,  66,  50
    flags: u8,
    rate: &'static [u16;16],
    load_counter: u8,
    sample_address: u8, // actual address is 0xC000 + sample_address*64
    sample_length: u8, // actual length is sample_length*16 + 1
}

impl Memory for Dmc {
    fn read(&mut self, address: u16) -> u8 {
        panic!("DMC registers are read-only! Attempted to read {:#x}", address);
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x4010 => self.flags = value & 0xCF, // ignore bits 4 and 5
            0x4011 => self.load_counter = value & 0x7F, // ignore bit 7
            0x4012 => self.sample_address = value,
            0x4013 => self.sample_length = value,
            _ => panic!("Invalid DMC address (address: {:#x}, value: {})", address, value),
        }
    }
}

impl Dmc {
    pub fn new() -> Dmc {
        Dmc {
            flags: 0,
            rate: &NTSC_RATE, // TEMPORARILY HARDCODED TO NTSC
            load_counter: 0,
            sample_address: 0,
            sample_length: 0,
        }
    }

    pub fn emulate_cycle() {

    }

    pub fn output() {

    }
}


