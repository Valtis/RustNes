use apu::envelope::Envelope;
use apu::timer::{Timer, TimerCycle};
use apu::length_counter::LengthCounter;
use apu::sweep::{Sweep, Complement, SweepCycle};
use memory::Memory;


/* duty cycles for the square wave
   for example, duty cycle 1 generates the following square wave:
     _ _             _ _
    |   |           |   |
    |   |           |   |
   _|   |_ _ _ _ _ _|   |_ _ _ _ _
   0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 .......
*/
static DUTY_CYCLES: [[u8; 8]; 4] = [
	[0, 1, 0, 0, 0, 0, 0, 0],
	[0, 1, 1, 0, 0, 0, 0, 0],
	[0, 1, 1, 1, 1, 0, 0, 0],
	[1, 0, 0, 1, 1, 1, 1, 1],
];

// selected duty cycle and the current position
struct Duty {
	duty_cycle: usize,
	duty_position: usize,
}

impl Duty {
    fn new() ->  Duty {
         Duty { duty_cycle: 0, duty_position: 0 }
    }

    fn cycle(&mut self) {
        if self.duty_position > 0 {
            self.duty_position -= 1;
        } else {
            self.duty_position = 7;
        }
    }
}

pub struct PulseChannel {
	duty: Duty,
	length_counter: LengthCounter,
	timer: Timer,
	pub envelope: Envelope,
    pub sweep: Sweep,
    enabled: bool,
}

impl Memory for PulseChannel {

    fn read(&mut self, address: u16) ->  u8 {
        panic!("Invalid read attempt of pulse channel register {}", address);
    }

    fn write(&mut self, address: u16, value: u8) {
        if address == 0x4000 || address == 0x4004 {
            let duty_cycle = (0b1100_0000 & value) >> 6;
            let length_counter_halt = (0b0010_0000 & value) != 0;
            let constant_volume_envelope_flag = (0b0001_0000 & value) != 0;
            let volume_envelope_divider_period = (0b0000_1111 & value);

            self.duty.duty_cycle = duty_cycle as usize;
            self.length_counter.halt(length_counter_halt);

            self.envelope.loop_flag(length_counter_halt);
            self.envelope.set_constant_volume(constant_volume_envelope_flag);
            self.envelope.set_constant_volume_or_envelope_period(
                volume_envelope_divider_period);
        } else if address == 0x4001 || address == 0x4005 {
            let sweep_enable = (0b1000_0000 & value) != 0;
            let divider_period = (0b0111_0000 & value) >> 4;
            let negate_flag = (0b0000_1000 & value) != 0;
            let shift_count = (0b0000_0111 & value);

            self.sweep.enabled = sweep_enable;
            self.sweep.length = divider_period + 1;
            self.sweep.negate = negate_flag;
            self.sweep.shift = shift_count;
            self.sweep.reload = true;

        } else if address == 0x04002 || address == 0x4006 {
            let timer_low_bits = value;
            self.timer.set_low_bits(value);
        } else if address == 0x4003 || address == 0x4007 {
            let length_counter_load = (0b1111_1000 & value) >> 3;
            let timer_high_bits = (0b0000_0111 & value);

            self.length_counter.load(length_counter_load);

            self.envelope.restart_envelope();
            self.timer.set_high_bits(timer_high_bits);
            self.duty.duty_position = 0;

        } else {
            panic!("Invald write to pulse channel address {:0x}",
                address);
        }
    }
}

impl PulseChannel {
    pub fn new(complement: Complement) -> PulseChannel {
        PulseChannel {
            duty: Duty::new(),
            length_counter: LengthCounter::new(),
            timer: Timer::new(),
            envelope: Envelope::new(),
            sweep: Sweep::new(complement),
            enabled: false,
        }
    }

    pub fn enable_channel(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.length_counter.enable(enabled);
    }

    pub fn cycle_timer(&mut self) {
        if self.timer.cycle() == TimerCycle::ZeroCycle {
            self.duty.cycle();
        }
    }

    pub fn cycle_envelope(&mut self) {
        self.envelope.cycle();
    }

    pub fn cycle_length_counter(&mut self) {
        self.length_counter.cycle();
    }

    pub fn cycle_sweep_unit(&mut self) {
        if self.sweep.cycle() == SweepCycle::ZeroCycle {
            let change = self.sweep.sweep_amount(self.timer.length);

            if change > 2047 {
                return;
            }

            self.timer.length = (self.timer.length as i16 + change) as u16
                & 0b0000_0111_1111_1111;
        }
    }

    pub fn length_counter_nonzero(&self) -> bool {
        self.length_counter.counter > 0
    }

    pub fn output(&self) -> f64 {
        if !self.enabled
            || self.length_counter.silenced()
            || self.timer.length < 8
            || self.sweep.last_change > 2047 {
            return 0.0;
        }

        let volume = self.envelope.volume() as f64;
        let duty_val = DUTY_CYCLES[self.duty.duty_cycle][self.duty.duty_position];
        volume * duty_val as f64
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_channel() -> PulseChannel {
        let mut channel = PulseChannel::new(Complement::Two);
        channel.duty = Duty::new();
        channel.duty.duty_cycle = 0;
        channel.duty.duty_position = 1;
        channel.timer.set_period(20);
        channel.enable_channel(true);
        channel.envelope.set_constant_volume(true);
        channel.envelope.set_constant_volume_or_envelope_period(5);
        assert_eq!(
            DUTY_CYCLES[channel.duty.duty_cycle][channel.duty.duty_position],
            1);

        channel
    }

    #[test]
    fn output_is_zero_if_length_counter_silences_channel() {
        let mut channel = create_test_channel();
        channel.length_counter.counter = 5;
        channel.length_counter.counter = 0;
        assert_eq!(channel.output(), 0.0);
    }

    #[test]
    fn output_is_envelope_value_if_length_counter_does_not_silence_channel() {
        let mut channel = create_test_channel();
        channel.length_counter.counter = 5;
        channel.length_counter.counter = 2;
        assert_eq!(channel.output(), 5.0);
    }

    #[test]
    fn writing_to_0x4003_loads_length_counter() {
        let mut channel = create_test_channel();
        let val = (6 & 0b0001_1111) << 3;
        channel.write(0x4003, val);
        assert_eq!(channel.length_counter.length, 80);
        assert_eq!(channel.length_counter.counter, 80);
    }

    #[test]
    fn writing_to_0x4007_loads_length_counter() {
        let mut channel = create_test_channel();
        let val = (8 & 0b0001_1111) << 3;
        channel.write(0x4007, val);
        assert_eq!(channel.length_counter.length, 160);
        assert_eq!(channel.length_counter.counter, 160);
    }

    #[test]
    fn writing_to_0x4000_sets_length_counter_halt_flag() {
        let mut channel = create_test_channel();
        assert!(!channel.length_counter.halted());
        let val = 0b0010_0000;
        channel.write(0x4000, val);
        assert!(channel.length_counter.halted());
    }

    #[test]
    fn writing_to_0x4004_sets_length_counter_halt_flag() {
        let mut channel = create_test_channel();
        assert!(!channel.length_counter.halted());
        let val = 0b0010_0000;
        channel.write(0x4004, val);
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
}

