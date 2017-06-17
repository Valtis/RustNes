use memory::Memory;

use apu::timer::{Timer, TimerCycle};
use apu::envelope::Envelope;
use apu::length_counter::LengthCounter;

static NTSC_RATE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160,
    202, 254, 380, 508, 762, 1016, 2034, 4068
];

static PAL_RATE: [u16; 16] = [
    4, 8, 14, 30, 60, 88, 118, 148,
    188, 236, 354, 472, 708,  944, 1890, 3778
];

pub struct NoiseChannel {
    enabled: bool,
    length_counter: LengthCounter,
    envelope: Envelope,
    timer: Timer,
    mode_flag: bool,
    shift_register: u16,
}


impl Memory for NoiseChannel {

    fn read(&mut self, address: u16) ->  u8 {
        panic!("Invalid read attempt of noise channel register {:0x}",
            address);
    }

    fn write(&mut self, address: u16, value: u8) {
        if address == 0x400C {
            let length_counter_halt = (0b0010_0000 & value) != 0;
            let constant_volume_envelope_flag = (0b0001_0000 & value) != 0;
            let volume_divider_period = (0b0000_1111 & value);

            self.envelope.set_constant_volume(constant_volume_envelope_flag);
            self.
                envelope.
                set_constant_volume_or_envelope_period(volume_divider_period);


            self.length_counter.halt(length_counter_halt);
        } else if address == 0x400D {
            /* Unused */
        } else if address == 0x400E {
            let mode_flag = (0b1000_0000 & value) != 0;
            let rate_index = (0b0000_1111 & value);

            self.mode_flag = mode_flag;
            // FIXME: Select NTSC/PAL rate correctly
            let rate = NTSC_RATE[rate_index as usize];
            self.timer.set_period(rate);

        } else if address == 0x400F {
            let length_counter_load = (0b1111_1000 & value) >> 3;
            self.length_counter.load(length_counter_load);

            self.envelope.restart_envelope();
        } else {
            panic!("Invalid write to noise channel address {:0x}",
                address);
        }
    }
}

impl NoiseChannel {

    pub fn new() -> NoiseChannel {
        NoiseChannel {
            enabled: false,
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),
            timer: Timer::new(),
            mode_flag: false,
            shift_register: 1, // value of reg after power up is 1
        }
    }

    pub fn cycle_envelope(&mut self) {
        self.envelope.cycle();
    }

    pub fn cycle_length_counter(&mut self) {
        self.length_counter.cycle();
    }

    pub fn cycle_timer(&mut self) {
        if self.timer.cycle() == TimerCycle::ZeroCycle {

            let bit_1 = (0b0000_0000_0000_0001 & self.shift_register);

            let bit_2 = if self.mode_flag {
                (0b0000_0000_0100_0000 & self.shift_register) >> 6
            } else {
                (0b0000_0000_0000_0010 & self.shift_register) >> 1
            };
            self.shift_register = self.shift_register >> 1;
            self.shift_register = self.shift_register | (bit_1 ^ bit_2) << 14;
        }
    }

    pub fn length_counter_nonzero(&self) -> bool {
        self.length_counter.counter > 0
    }


    pub fn enable_channel(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn output(&self) -> f64 {
        if !self.enabled
            || self.length_counter.silenced()
            || (0b0000_0000_0000_0001 & self.shift_register) != 0 {
            return 0.0;
        }

        self.envelope.volume() as f64
    }
}