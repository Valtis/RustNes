#[derive(PartialEq)]
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
}
