use memory::Memory;

use std::cell::RefCell;
use std::rc::Rc;

// how many cpu cycles per single dmc output change
static NTSC_RATE : [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214,
    190, 160, 142, 128, 106,  84,  72,  54
];

static PAL_RATE : [u16; 16] = [
    398, 354, 316, 298, 276, 236, 210, 198,
    176, 148, 132, 118,  98,  78,  66,  50
];

struct Reader<'a> {
    sample_address: u16,
    sample_length: u16,
    current_addess: u16,
    current_length: u16,
    loop_flag: bool,
    interrupt_enabled: bool,
    interrupt_flag: bool,
    memory: Option<Rc<RefCell<Box<Memory + 'a>>>>,
}

impl<'a> Reader<'a> {
    fn new() -> Reader<'a> {
        Reader {
            sample_address: 0,
            sample_length: 0,
            current_length: 0,
            current_addess: 0,
            loop_flag: false,
            interrupt_enabled: false,
            interrupt_flag: false,
            memory: None,
        }
    }

    fn get_output_buffer(&mut self) -> Option<u8> {
        if self.current_length == 0 {
            if self.loop_flag {
                self.current_length = self.sample_length;
                self.current_addess = self.sample_address;
            } else if self.interrupt_enabled {
                self.interrupt_flag = true;
                return None;
            }
        }

        // FIXME: CPU/PPU stalls not implemented. May affect timing
        let out = if let Some(ref memory) = self.memory {
            memory.borrow_mut().read(self.current_addess)
        } else {
            panic!("Memory bus unexpectedly none");
        };


        self.current_length -= 1;
        if self.current_addess == 0xFFFF {
            self.current_addess = 0x8000;
        } else {
            self.current_addess += 1;
        }

        return Some(out);
    }
}

struct Output {
    buffer: Option<u8>,
    bits_remaining: u8,
    output_level: u8
}

impl Output {
    fn new() -> Output {
        Output {
            buffer: None,
            bits_remaining: 0,
            output_level: 0,
        }
    }

    fn cycle(&mut self, reader: &mut Reader, enabled: bool) {
        if self.bits_remaining > 0 {
            if let Some(buffer) = self.buffer {
                let mask = 0b0000_0001;

                if (mask & buffer) != 0
                    && self.output_level <= 125 {
                    self.output_level += 2;
                } else if (mask & buffer) == 0
                    && self.output_level >= 2 {
                    self.output_level -= 2;
                }
                self.bits_remaining -= 1;
                self.buffer = Some(buffer >> 1);

            }
        } else {
            if enabled {
                self.bits_remaining = 8;
                self.buffer = reader.get_output_buffer();
            }
        }
    }
}

pub struct DmcChannel<'a> {
    enabled: bool,
    rate: u16,
    counter: u16,
    reader: Reader<'a>,
    output: Output,
}

impl<'a> Memory for DmcChannel<'a> {

    fn read(&mut self, address: u16) ->  u8 {
        panic!("Invalid read attempt of dmc channel register {:0x}",
            address);
    }

    fn write(&mut self, address: u16, value: u8) {
        if address == 0x4010 {
            self.reader.interrupt_enabled = (0b1000_0000 & value) != 0;
            if !self.reader.interrupt_enabled {
                self.reader.interrupt_flag = false;
            }
            self.reader.loop_flag = (0b0100_0000 & value) != 0;
            // FIXME: Properly select NTSC/PAL rates
            self.rate = NTSC_RATE[(0b0000_1111 & value) as usize];
        } else if address == 0x4011 {
            self.output.output_level = (0b0111_1111 & value);
        } else if address == 0x4012 {
            self.reader.sample_address = 0xC000 + 64 * value as u16;
        } else if address == 0x4013 {
            self.reader.sample_length = value as u16 * 16 + 1;
        } else {
            panic!("Invalid write to dmc channel address {:0x}",
                address);
        }
    }
}

impl<'a> DmcChannel<'a> {
    pub fn new() -> DmcChannel<'a> {
        DmcChannel {
            enabled: false,
            rate: 0,
            counter: 0,
            reader: Reader::new(),
            output: Output::new(),
        }
    }

    pub fn enable_channel(&mut self, enable: bool) {
        self.enabled = enable;
    }

    pub fn cycle_timer(&mut self) {
        if !self.enabled {
            self.reader.current_length = 0;
        }

        if self.counter == self.rate {
            self.output.cycle(&mut self.reader, self.enabled);
            self.counter = 0;
        } else {
            self.counter += 2; // 1 apu cycle = 2 cpu cycles
        }
    }

    pub fn output(&self) -> f64 {
        if let Some(_) = self.output.buffer {
            self.output.output_level as f64
        } else {
            0.0
        }
    }

    pub fn pending_interrupt(&self) -> bool {
        self.reader.interrupt_flag
    }

    pub fn clear_interrupt(&mut self) {
        self.reader.interrupt_flag = false;
    }

    pub fn set_memory(&mut self, mem: Rc<RefCell<Box<Memory + 'a>>>) {
        self.reader.memory = Some(mem);
    }

    pub fn active(&self) -> bool {
        return self.reader.current_length > 0
    }
}