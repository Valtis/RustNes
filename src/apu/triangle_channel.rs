use memory::Memory;
use apu::timer::{Timer, TimerCycle};
use apu::length_counter::LengthCounter;
use apu::linear_counter::LinearCounter;

static CYCLE : [u8; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
    0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

pub struct TriangleChannel {
    timer: Timer,
    length_counter: LengthCounter,
    pub linear_counter: LinearCounter,
    sequence: usize,
    enabled: bool,
}

impl Memory for TriangleChannel {

    fn read(&mut self, address: u16) ->  u8 {
        panic!("Invalid read attempt of triangle channel register {:0x}",
            address);
    }

    fn write(&mut self, address: u16, value: u8) {
        if address == 0x4008 {
            let control_flag = (0b1000_0000 & value) != 0;
            self.linear_counter.control = control_flag;

            self.length_counter.halt(control_flag);

            let counter_reload = (0b0111_1111 & value);
            self.linear_counter.length = counter_reload;
        } else if address == 0x4009 {
            /* unused */
        } else if address == 0x400A {
            let timer_low_bits = value;
            self.timer.set_low_bits(timer_low_bits);
        } else if address == 0x0400B {

            let length_counter_load = (0b1111_1000 & value) >> 3;
            let timer_high_bits = (0b0000_0111 & value);

            self.length_counter.load(length_counter_load);
            self.timer.set_high_bits(timer_high_bits);

            self.linear_counter.reload = true;
        } else {
            panic!("Invalid write to triangle channel address {:0x}",
                address);
        }
    }
}

impl TriangleChannel {

    pub fn new() -> TriangleChannel {
        TriangleChannel {
            timer: Timer::new(),
            length_counter: LengthCounter::new(),
            linear_counter: LinearCounter::new(),
            sequence: 0,
            enabled: false,
        }
    }

    pub fn enable_channel(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.enable(enabled);
    }


    pub fn output(&self) -> f64 {
        if !self.enabled
            || self.length_counter.silenced()
            || self.linear_counter.silenced() {
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

    pub fn length_counter_nonzero(&self) -> bool {
        self.length_counter.counter > 0
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_channel() -> TriangleChannel {
        let mut channel = TriangleChannel::new();
        channel.timer.set_period(20);
        channel.enable_channel(true);
        channel.linear_counter.counter = 5;
        channel.linear_counter.length = 5;
        channel.length_counter.length = 5;
        channel.length_counter.counter = 5;
        channel.sequence = 5;
        channel
    }

    #[test]
    fn cycling_channel_counter_plus_one_times_advances_sequence() {
        let mut channel = create_test_channel();
        channel.timer.length = 4;
        channel.timer.counter = 4;

        channel.cycle_timer(); // 4 -> 3
        channel.cycle_timer(); // 3 -> 2
        channel.cycle_timer(); // 2 -> 1
        channel.cycle_timer(); // 1 -> 0
        channel.cycle_timer(); // 0 -> 4, sequence advances
        assert_eq!(channel.sequence, 6);
    }

    #[test]
    fn cycling_channel_timer_length_times_does_not_advance_sequence() {
        let mut channel = create_test_channel();
        channel.timer.length = 4;
        channel.timer.counter = 4;

        channel.cycle_timer();
        channel.cycle_timer();
        channel.cycle_timer();
        channel.cycle_timer();

        assert_eq!(channel.sequence, 5);
    }

    #[test]
    fn output_is_zero_if_length_counter_silences_channel() {
        let mut channel = create_test_channel();
        channel.length_counter.counter = 0;
        assert_eq!(channel.output(), 0.0);
    }

    #[test]
    fn output_is_zero_if_linear_counter_silences_channel() {
         let mut channel = create_test_channel();
        channel.linear_counter.counter = 0;
        assert_eq!(channel.output(), 0.0);
    }

    #[test]
    fn output_is_sequenced_value_if_channel_is_not_silenced() {
        let mut channel = create_test_channel();
        assert_eq!(channel.output(), 10.0);
    }

    #[test]
    fn writing_to_0x400B_loads_length_counter() {
        let mut channel = create_test_channel();
        let val = (6 & 0b0001_1111) << 3;
        channel.write(0x400B, val);
        assert_eq!(channel.length_counter.length, 80);
        assert_eq!(channel.length_counter.counter, 80);
    }

    #[test]
    fn writing_to_0x4008_sets_length_counter_halt_flag() {
        let mut channel = create_test_channel();
        assert!(!channel.length_counter.halted());
        let val = 0b1000_0000;
        channel.write(0x4008, val);
        assert!(channel.length_counter.halted());
    }

    #[test]
    fn enabling_channel_enables_length_counter() {
        let mut channel = create_test_channel();
        channel.enable_channel(false);
        assert!(!channel.length_counter.enabled());
        channel.enable_channel(true);
        assert!(channel.length_counter.enabled());
    }

    #[test]
    fn disabling_channel_disables_length_counter() {
        let mut channel = create_test_channel();
        channel.enable_channel(true);
        assert!(channel.length_counter.enabled());
        channel.enable_channel(false);
        assert!(!channel.length_counter.enabled());
    }

    #[test]
    fn cycle_length_counter_method_actually_cycles_length_counter() {
        let mut channel = create_test_channel();
        channel.length_counter.halt(false);
        channel.length_counter.length = 4;
        channel.length_counter.counter = 4;
        channel.cycle_length_counter();
        assert_eq!(channel.length_counter.counter, 3);
    }

    #[test]
    fn writing_to_0x4008_sets_linear_counter_control_and_value() {
        let mut channel = create_test_channel();
        channel.linear_counter.control = false;

        channel.write(0x4008, 0b1001_0001);
        assert!(channel.linear_counter.control);
        assert_eq!(channel.linear_counter.length, 17);
    }

    #[test]
    fn writing_to_0x400B_sets_linear_counter_reload_flag() {
        let mut channel = create_test_channel();
        channel.linear_counter.reload = false;

        channel.write(0x400B, 0);
        assert!(channel.linear_counter.reload);
    }

    #[test]
    fn cycle_linear_counter_actually_cycles_linear_counter() {
        let mut channel = create_test_channel();
        channel.linear_counter.counter = 5;
        channel.cycle_linear_counter();
        assert_eq!(channel.linear_counter.counter, 4);
    }

    #[test]
    fn write_to_0x400A_sets_timer_low_bits() {
        let mut channel = create_test_channel();
        channel.timer.length = 0b0000_0101_0110_1100;
        channel.write(0x400A, 0b1000_1010);
        assert_eq!(channel.timer.length, 0b0000_0101_1000_1010);
    }

    #[test]
    fn write_to_0x400B_sets_timer_high_bits() {
        let mut channel = create_test_channel();
        channel.timer.length = 0b0000_0101_0110_1100;
        channel.write(0x400B, 0b0000_0010);
        assert_eq!(channel.timer.length, 0b0000_0010_0110_1100);
    }

}

