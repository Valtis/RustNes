use apu::envelope::Envelope;
use apu::timer::{Timer, TimerCycle};
use apu::length_counter::LengthCounter;
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

#[derive(PartialEq)]
pub enum Complement {
    One,
    Two,
}

#[derive(PartialEq)]
enum SweepCycle {
    ZeroCycle,
    NormalCycle
}

struct Sweep {
    counter: u8,
    length: u8,
    shift: u8,
    enabled: bool,
    negate: bool,
    reload: bool,
    complement: Complement
}

impl Sweep {
    fn new(complement: Complement) -> Sweep {
        Sweep {
            counter: 0,
            length: 0,
            shift: 0,
            enabled: false,
            negate: false,
            reload: false,
            complement: complement,
        }
    }

    fn cycle(&mut self) -> SweepCycle {

        if self.reload {
            let old_val = self.counter;
            self.counter = self.length;

            if old_val == 0 && self.enabled {
                return SweepCycle::ZeroCycle;
            }

            return SweepCycle::NormalCycle;
        }

        if self.counter > 0 && !self.reload {
            self.counter -= 1;
        } else if self.counter == 0 && !self.reload && self.enabled  {
            self.counter = self.length;
            return SweepCycle::ZeroCycle;
        }

        SweepCycle::NormalCycle
    }

    fn sweep_amount(&self, base: u16) -> i16 {
        let mut sweep = (base << self.shift) as i16;
        if self.negate {
            if self.complement == Complement::One {
                return -sweep - 1;
            } else {
                return -sweep;
            }
        }
        sweep
    }
}

// selected duty cycle and the current position
struct Duty {
	duty_cycle: usize,
	duty_position: usize,
}

impl Duty {
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
	envelope: Envelope,
    sweep: Sweep,
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
            // TODO: reset phase

        } else {
            unimplemented!();
        }
    }
}

impl PulseChannel {
    pub fn new(complement: Complement) -> PulseChannel {
        PulseChannel {
            duty: Duty { duty_cycle: 0, duty_position: 0 },
            length_counter: LengthCounter::new(0),
            timer: Timer::new(),
            envelope: Envelope::new(),
            sweep: Sweep::new(complement),
            enabled: false,
        }
    }

    pub fn enable_channel(&mut self, enabled: bool) {
        self.enabled = enabled;
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
            self.timer.length = (self.timer.length as i16 + change) as u16
                & 0b0000_0111_1111_1111;
        }
    }

    pub fn output(&self) -> f64 {
        if !self.enabled
            || self.length_counter.silenced()
            || self.timer.length < 8
            || self.sweep.length < 8 {
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

	fn create_test_length_counter() -> LengthCounter {
		LengthCounter::new(50)
	}

}