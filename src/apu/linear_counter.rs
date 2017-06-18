

pub struct LinearCounter {
    pub counter: u8,
    pub length: u8,
    pub reload: bool,
    pub control: bool
}

impl LinearCounter {
    pub fn new() -> LinearCounter {
        LinearCounter {
            counter: 0,
            length: 0,
            reload: false,
            control: false,
        }
    }

    pub fn cycle(&mut self) {
        if self.reload {
            self.counter = self.length;
        } else if self.counter > 0 {
            self.counter -= 1;
        }

        if !self.control {
            self.reload = false;
        }
    }

    pub fn silenced(&self) -> bool {
        self.counter == 0
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_counter_does_not_silence_channel_if_counter_has_nonzero_value() {
        let mut counter = LinearCounter::new();
        counter.counter = 5;
        counter.length = 5;
        assert!(!counter.silenced());
    }

    #[test]
    fn linear_counter_silences_channel_if_counter_is_zero() {
        let mut counter = LinearCounter::new();
        counter.counter = 0;
        counter.length = 5;
        assert!(counter.silenced());
    }

    #[test]
    fn linear_counter_value_is_reloaded_if_reload_flag_is_set() {
        let mut counter = LinearCounter::new();
        counter.reload = true;
        counter.counter = 5;
        counter.length = 20;
        counter.cycle();
        assert_eq!(counter.counter, 20);
    }

    #[test]
    fn linear_counter_reload_flag_is_not_cleared_if_control_is_set() {
        let mut counter = LinearCounter::new();
        counter.reload = true;
        counter.control = true;
        counter.cycle();
        assert!(counter.reload);
    }

    #[test]
    fn linear_counter_reload_flag_is_cleared_if_control_is_not_set() {
        let mut counter = LinearCounter::new();
        counter.reload = true;
        counter.control = false;
        counter.cycle();
        assert!(!counter.reload);
    }

    #[test]
    fn counter_is_decremented_if_reload_is_not_set() {
        let mut counter = LinearCounter::new();
        counter.reload = false;
        counter.counter = 5;
        counter.length = 20;
        counter.cycle();
        assert_eq!(counter.counter, 4);
    }

    #[test]
    fn counter_is_not_decremented_if_it_is_zero() {
        let mut counter = LinearCounter::new();
        counter.reload = false;
        counter.counter = 0;
        counter.length = 20;
        counter.cycle();
        assert_eq!(counter.counter, 0);
    }

}