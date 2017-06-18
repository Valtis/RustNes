#[derive(PartialEq, Debug)]
pub enum TimerCycle {
    ZeroCycle,
    NormalCycle,
}

pub struct Timer {
    pub length: u16,
    pub counter: u16,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { length: 0, counter: 0 }
    }

    pub fn cycle(&mut self) -> TimerCycle {
        if self.counter > 0 {
            self.counter -= 1;
            TimerCycle::NormalCycle
        } else {
            self.counter = self.length;
            TimerCycle::ZeroCycle
        }
    }


    pub fn set_high_bits(&mut self, value: u8) {
        self.length = (self.length & 0b0000_0000_1111_1111) |
            ((value as u16) << 8);
    }

    pub fn set_low_bits(&mut self, value: u8) {
        self.length = (self.length & 0b0000_0111_0000_0000)
            | value as u16;
    }

    pub fn set_period(&mut self, period: u16) {
        self.length = period;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_is_clocked_when_it_goes_from_zero_to_length() {
        let mut timer = Timer::new();
        timer.length = 4;
        timer.counter = 4;

        assert_eq!(timer.cycle(), TimerCycle::NormalCycle);
        assert_eq!(timer.cycle(), TimerCycle::NormalCycle);
        assert_eq!(timer.cycle(), TimerCycle::NormalCycle);
        assert_eq!(timer.cycle(), TimerCycle::NormalCycle);
        assert_eq!(timer.cycle(), TimerCycle::ZeroCycle);
    }

    #[test]
    fn timer_value_is_reloaded_when_cycled_at_zero() {
        let mut timer = Timer::new();
        timer.length = 4;
        timer.counter = 0;
        timer.cycle();
        assert_eq!(timer.counter, 4);
    }

    #[test]
    fn low_bits_are_set_correctly() {
       let mut timer = Timer::new();
       timer.length = 0b0000_0101_1001_1110;
       timer.set_low_bits(0b0110_0101);
       assert_eq!(timer.length, 0b0000_0101_0110_0101);
    }

    #[test]
    fn high_bits_are_set_correctly() {
       let mut timer = Timer::new();
       timer.length = 0b0000_0101_1001_1110;
       timer.set_high_bits(0b0000_0110);
       assert_eq!(timer.length, 0b0000_0110_1001_1110);
    }
}
