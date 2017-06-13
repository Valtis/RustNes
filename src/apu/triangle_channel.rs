use memory::Memory;
use apu::timer::{Timer, TimerCycle};
use apu::length_counter::LengthCounter;


static CYCLE : [u8; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
    0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

pub struct LinearCounter {
    count: u8,
    length: u8,
    reload: bool,
    control: bool
}

impl LinearCounter {
    fn new() -> LinearCounter {
        LinearCounter {
            count: 0,
            length: 0,
            reload: false,
            control: false,
        }
    }

    fn cycle(&mut self) {
        if self.reload {
            self.count = self.length;
        } else if self.count > 0 {
            self.count -= 1;
        }

        if !self.control {
            self.reload = false;
        }
    }
}

pub struct TriangleChannel {
    timer: Timer,
    length_counter: LengthCounter,
    linear_counter: LinearCounter,
    sequence: usize,
    enabled: bool,
}

impl Memory for TriangleChannel {

    fn read(&mut self, address: u16) ->  u8 {
        panic!("Invalid read attempt of triangle channel register {}", address);
    }

    fn write(&mut self, address: u16, value: u8) {
        if address == 0x4008 {
            let control_flag = (0b1000_0000 & value) != 0;
            self.linear_counter.control = control_flag;

            self.length_counter.halt(control_flag);

            let counter_reload = (0b0111_1111 & value);
            self.linear_counter.length = counter_reload;

        } else if address == 0x400A {
            let timer_low_bits = value;
            self.timer.set_low_bits(timer_low_bits);
        } else if address == 0x0400B {

            let length_counter_load = (0b1111_1000 & value) >> 3;
            let timer_high_bits = (0b0000_0111 & value);

            self.length_counter.load(length_counter_load);
            self.timer.set_high_bits(timer_high_bits);

            self.linear_counter.reload = true;
        }
    }
}

impl TriangleChannel {

    pub fn new() -> TriangleChannel {
        TriangleChannel {
            timer: Timer::new(),
            length_counter: LengthCounter::new(0),
            linear_counter: LinearCounter::new(),
            sequence: 0,
            enabled: false,
        }
    }

    pub fn enable_channel(&mut self, enabled: bool) {
        self.enabled = enabled;
    }


    pub fn output(&self) -> f64 {
        if !self.enabled
            || self.length_counter.silenced()
            || self.linear_counter.count == 0  {
            return 0.0;
        }

        CYCLE[self.sequence] as f64
    }

    pub fn cycle_timer(&mut self) {
        if self.timer.cycle() == TimerCycle::ZeroCycle {
            self.sequence += 1;
            if self.sequence >= CYCLE.len() {
                self.sequence = 0;
            }
        }
    }

    pub fn cycle_linear_counter(&mut self) {
        self.linear_counter.cycle();
    }

    pub fn cycle_length_counter(&mut self) {
        self.length_counter.cycle();
    }

}