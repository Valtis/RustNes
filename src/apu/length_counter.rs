// counts from length to zero. Channel is silenced on zero



static LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];


pub struct LengthCounter {
    pub length: u8,
    pub counter: u8,
    halted: bool,
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter {
            length: 0,
            counter: 0,
            halted: false,
        }
    }

    pub fn cycle(&mut self) {
        if !self.halted && self.counter != 0 {
            self.counter -= 1;
        }
    }

    pub fn silenced(&self) -> bool {
        !self.halted && self.counter == 0
    }

    pub fn load(&mut self, index: u8) {
        self.counter = LENGTH_COUNTER_TABLE[index as usize];
        self.length = LENGTH_COUNTER_TABLE[index as usize];
    }

    pub fn halt(&mut self, halt: bool) {
        self.halted = halt;
    }
}
