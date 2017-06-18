// counts from length to zero. Channel is silenced on zero



static LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];


pub struct LengthCounter {
    pub length: u8,
    pub counter: u8,
    halted: bool,
    enabled: bool
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter {
            length: 0,
            counter: 0,
            halted: false,
            enabled: true
        }
    }

    pub fn cycle(&mut self) {
        if !self.halted && self.counter != 0 {
            self.counter -= 1;
        }
    }

    pub fn silenced(&self) -> bool {
        self.counter == 0
    }

    pub fn load(&mut self, index: u8) {
        if self.enabled {
            self.counter = LENGTH_COUNTER_TABLE[index as usize];
            self.length = LENGTH_COUNTER_TABLE[index as usize];
        }
    }

    pub fn halt(&mut self, halt: bool) {
        self.halted = halt;
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn enable(&mut self, enabled: bool) {
        if !enabled {
            self.counter = 0;
            self.length = 0;
        }

        self.enabled = enabled;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {

    use super::LengthCounter;

    #[test]
    fn length_counter_is_set_zero_when_disabled() {
        let mut counter = LengthCounter::new();
        counter.counter = 1;
        counter.length = 1;
        counter.enable(false);
        assert_eq!(counter.counter, 0);
        assert_eq!(counter.length, 0);
    }

    #[test]
    fn no_value_is_loaded_into_counter_if_counter_is_disabled() {
        let mut counter = LengthCounter::new();
        counter.enable(false);
        counter.load(4);
        assert_eq!(counter.counter, 0);
        assert_eq!(counter.length, 0);
    }

    #[test]
    fn value_is_loaded_into_counter_if_counter_is_enabled() {
        let mut counter = LengthCounter::new();
        counter.enable(true);
        counter.load(4);
        assert_eq!(counter.counter, 40);
        assert_eq!(counter.length, 40);
    }

    #[test]
    fn length_counter_is_not_silent_if_counter_has_nonzero_value() {
        let mut counter = LengthCounter::new();
        counter.counter = 1;
        assert!(!counter.silenced());
    }

    #[test]
    fn length_counter_is_silent_if_counter_has_zero_value() {
        let mut counter = LengthCounter::new();
        counter.counter = 0;
        assert!(counter.silenced());
    }

    #[test]
    fn length_counter_is_silent_after_cycling_to_zero_value() {
        let mut counter = LengthCounter::new();
        counter.counter = 1;
        counter.cycle();
        assert!(counter.silenced());
    }

    #[test]
    fn length_counter_is_not_cycled_when_counter_is_halted() {
        let mut counter = LengthCounter::new();
        counter.counter = 1;
        counter.halt(true);
        counter.cycle();
        assert_eq!(counter.counter, 1);
    }

    #[test]
    fn length_counter_is_cycled_when_halt_flag_is_cleared() {
        let mut counter = LengthCounter::new();
        counter.counter = 1;
        counter.halt(true);
        counter.cycle();
        counter.halt(false);
        counter.cycle();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn counter_is_not_cycled_when_already_at_zero() {
        let mut counter = LengthCounter::new();
        counter.counter = 0;
        counter.cycle();
        assert_eq!(counter.counter, 0);
    }

    #[test]
    fn counter_is_loaded_with_correct_value_from_the_table() {
        let mut counter = LengthCounter::new();
        counter.counter = 0;

        counter.load(4);
        assert_eq!(counter.counter, 40);

        counter.load(1);
        assert_eq!(counter.counter, 254);


        counter.load(31);
        assert_eq!(counter.counter, 30);
    }
}
