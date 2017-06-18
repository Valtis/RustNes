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
    current_address: u16,
    current_length: u16,
    loop_flag: bool,
    interrupt_enabled: bool,
    interrupt_flag: bool,
    memory: Option<Rc<RefCell<Box<Memory + 'a>>>>,
    buffer: Option<u8>,
    delay_cpu: u8,
    borrow_read_workaround: Option<u16>,
}

impl<'a> Reader<'a> {
    fn new() -> Reader<'a> {
        Reader {
            sample_address: 0,
            sample_length: 0,
            current_length: 0,
            current_address: 0,
            loop_flag: false,
            interrupt_enabled: false,
            interrupt_flag: false,
            memory: None,
            buffer: None,
            delay_cpu: 0,
            borrow_read_workaround: None,
        }
    }

    fn enable(&mut self, enable: bool) {
        if enable && self.current_length == 0 {
            self.current_length = self.sample_length;
            self.current_address = self.sample_address;

            if self.buffer == None {
                self.get_output_buffer();
            }

        } else if !enable {
            self.current_length = 0;
        }
    }

    fn get_output_buffer(&mut self) -> Option<u8> {

        let out = self.buffer;
        self.buffer = None;

        if self.current_length > 0 {
            // FIXME: CPU stall duration varies, not always 4
            self.delay_cpu = 4;

            let new_buf = if let Some(ref memory) = self.memory {
                // borrow checker workaround. This may get invoked when
                // memory is being written into --> memory already borrowed
                // --> attempt to reborrow panics.
                // set a flag to read the value after memory is available again
                if let Some(_) = self.borrow_read_workaround {
                    panic!("Borrow workaround broke");
                }
                self.borrow_read_workaround = Some(self.current_address);
            } else {
                panic!("Memory bus unexpectedly None");
            };


            if self.current_address == 0xFFFF {
                self.current_address = 0x8000;
            } else {
                self.current_address += 1;
            }

            self.current_length -= 1;
            if self.current_length == 0 {
                if self.loop_flag {
                    self.current_length = self.sample_length;
                    self.current_address = self.sample_address
                } else if self.interrupt_enabled {
                    self.interrupt_flag = true;
                }
            }

            self.buffer = Some(0);
        } else {
            assert!(self.interrupt_flag || !self.loop_flag);
        }

        return out;
    }

    fn borrow_workaround(&mut self) {
        if let Some(addr) = self.borrow_read_workaround {

            if let Some(ref memory) = self.memory {
                let val = memory.borrow_mut().read(addr);
                self.buffer = Some(val);
            } else {
                panic!("Invariant violation in apu dmc mem read");
            };

            self.borrow_read_workaround = None;
        }
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
        if self.bits_remaining == 0 {
            self.buffer = reader.get_output_buffer();
            self.bits_remaining = 8;
        }

        if let Some(buffer) = self.buffer {
            let mask = 0b0000_0001;

            if (mask & buffer) != 0
                && self.output_level <= 125 {
                self.output_level += 2;
            } else if (mask & buffer) == 0
                && self.output_level >= 2 {
                self.output_level -= 2;
            }
            self.buffer = Some(buffer >> 1);
        }
        self.bits_remaining -= 1;
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
        self.reader.enable(enable);
    }

    pub fn cycle_timer(&mut self) {
        self.reader.borrow_workaround();

        if !self.enabled {
            return;
        }

        if self.counter == self.rate - 1 {
            self.output.cycle(&mut self.reader, self.enabled);
            self.counter = 0;
        } else {
            self.counter += 1;
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
        self.reader.current_length > 0
    }

    #[cfg(test)]
    pub fn dmc_rate(&self) -> u16 {
        self.rate
    }

    pub fn delay_cpu(&mut self) -> u8 {
        let out = self.reader.delay_cpu;
        self.reader.delay_cpu = 0;
        out
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;
    use std::rc::Rc;
    use std::cell::RefCell;

    struct MockMemory {

    }

    impl MockMemory {
        fn new() -> MockMemory {
            MockMemory {

            }
        }
    }

    impl Memory for MockMemory {
        fn read(&mut self, address: u16) -> u8 {
            0
        }

        fn write(&mut self, address: u16, value: u8) {

        }
    }

    fn create_test_dmc<'a>() -> DmcChannel<'a> {
        let mut channel = DmcChannel::new();
        let mem = Rc::new(
            RefCell::new(
                Box::new(MockMemory::new()) as Box<Memory>));
        channel.set_memory(mem);
        channel
    }



    // implements tests present in the various nes APU test roms

    fn delay_dmc(dmc: &mut DmcChannel, count: u16) {
        for _ in 0..dmc.rate*8*count {
            dmc.cycle_timer();
        }
    }

    // tests from dmc basics test rom by blargg
    #[test]
    fn channel_is_active_and_then_disabled_after_sample_ends() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x0F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);

        delay_dmc(&mut dmc, 10);
        assert!(dmc.active());

        delay_dmc(&mut dmc, 10);
        assert!(!dmc.active());
    }

    #[test]
    fn channel_reloads_length_from_0x4013_when_restarting_at_zero_length() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x0F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 20);
        // should be inactive at this point
        assert!(!dmc.active());

        dmc.enable_channel(false);
        dmc.enable_channel(true);
        // should have read one byte on enabling
        assert!(dmc.reader.current_length == 16);

        delay_dmc(&mut dmc, 10);
        assert!(dmc.active());
        delay_dmc(&mut dmc, 10);
        assert!(!dmc.active());
    }

    #[test]
    fn writing_0x10_to_0x4015_should_restart_dmc_if_sample_is_finished() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x0F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 20);
        assert!(!dmc.active());
        dmc.enable_channel(true);
        assert!(dmc.active());
    }

    #[test]
    fn writing_0x10_to_0x4015_should_not_affect_channel_if_sample_is_active() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x0F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 10);
        dmc.enable_channel(true);
        assert!(dmc.active());
        delay_dmc(&mut dmc, 10);
        assert!(!dmc.active());
    }

    #[test]
    fn writing_0x00_to_0x4015_should_stop_sample() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x0F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 10);
        dmc.enable_channel(false);
        assert!(!dmc.active());
    }

    #[test]
    fn writing_to_0x4013_should_not_affect_current_length() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x0F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        dmc.write(0x4013, 2); // next sample is 33 bytes
        delay_dmc(&mut dmc, 20);
        dmc.enable_channel(true); // start the 33 byte sample
        dmc.write(0x4013, 1); // next sample 17 bytes
        delay_dmc(&mut dmc, 30);
        assert!(dmc.active());
        delay_dmc(&mut dmc, 6);
        assert!(!dmc.active());
    }

    #[test]
    fn irq_flag_not_set_when_irq_disabled() {
       let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x0F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 20);
        assert!(!dmc.pending_interrupt());
    }

    #[test]
    fn irq_flag_is_set_when_sample_ends_and_loop_not_set_and_irq_enabled() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x8F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        assert!(!dmc.pending_interrupt());
        delay_dmc(&mut dmc, 20);
        assert!(dmc.pending_interrupt());
    }

    #[test]
    fn disabling_dmc_interrupt_clears_dmc_interrupt_flag() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x8F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        assert!(!dmc.pending_interrupt());
        delay_dmc(&mut dmc, 20);
        assert!(dmc.pending_interrupt());
        dmc.write(0x4010, 0x0F);
        assert!(!dmc.pending_interrupt());
        dmc.write(0x4010, 0x8F);
        assert!(!dmc.pending_interrupt());
    }

    #[test]
    fn looped_sample_ends_only_when_0x00_is_written_to_0x4015() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x4F);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 100);
        assert!(dmc.active());
        dmc.enable_channel(false);
        assert!(!dmc.active());
    }

    #[test]
    fn looped_sample_does_not_set_irq_flags() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0xCF);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 100);
        assert!(!dmc.pending_interrupt());
        dmc.enable_channel(false);
        assert!(!dmc.pending_interrupt());
    }

    #[test]
    fn clearing_loop_flag_and_setting_it_again_does_not_stop_loop() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0xCF);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 17*3 + 10);
        dmc.write(0x4010, 0x8F);
        dmc.write(0x4010, 0xCF);
        delay_dmc(&mut dmc, 100);
        assert!(dmc.active());
    }

    #[test]
    fn clearing_loop_flag_ends_sample_once_it_reaches_end() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0xCF);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 17*3 + 10);
        dmc.write(0x4010, 0x8F);
        assert!(!dmc.pending_interrupt());
        assert!(dmc.active());
        delay_dmc(&mut dmc, 10);
        assert!(dmc.pending_interrupt());
        assert!(!dmc.active());
    }

    #[test]
    fn looped_sample_should_reload_length_from_0x4013_when_it_reaches_end() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0xCF);
        dmc.write(0x4013, 1);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 17*3 + 10);
        dmc.write(0x4013, 2);
        delay_dmc(&mut dmc, 10);
        dmc.write(0x4010, 0x8F);
        delay_dmc(&mut dmc, 23);
        assert!(!dmc.pending_interrupt());
        assert!(dmc.active());
        delay_dmc(&mut dmc, 10);
        assert!(dmc.pending_interrupt());
        assert!(!dmc.active());
    }

    #[test]
    fn writing_0x00_into_0x4013_should_yield_one_byte_sample() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x8F);
        dmc.write(0x4013, 0);
        dmc.enable_channel(true);
        delay_dmc(&mut dmc, 4);
        assert!(!dmc.active());
    }

    #[test]
    fn one_byte_buffer_is_immediately_filled() {
        let mut dmc = create_test_dmc();
        dmc.write(0x4012, 0x100); // random mem address, not used here
        dmc.write(0x4010, 0x8F);
        dmc.write(0x4013, 0);
        dmc.enable_channel(true);

        assert!(dmc.pending_interrupt());
        assert!(!dmc.active());

        dmc.enable_channel(true);
        assert!(dmc.active());
        delay_dmc(&mut dmc, 4);
        assert!(!dmc.active());
    }
}