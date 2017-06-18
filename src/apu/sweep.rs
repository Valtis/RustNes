
#[derive(PartialEq)]
pub enum Complement {
    One,
    Two,
}

#[derive(PartialEq)]
pub enum SweepCycle {
    ZeroCycle,
    NormalCycle
}

pub struct Sweep {
    pub counter: u8,
    pub length: u8,
    pub shift: u8,
    pub enabled: bool,
    pub negate: bool,
    pub reload: bool,
    pub complement: Complement,
    pub last_change: u16,
}

impl Sweep {
    pub fn new(complement: Complement) -> Sweep {
        Sweep {
            counter: 0,
            length: 0,
            shift: 0,
            enabled: false,
            negate: false,
            reload: false,
            complement: complement,
            last_change: 0,
        }
    }

    pub fn cycle(&mut self) -> SweepCycle {

        if self.reload {
            self.reload = false;
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

    pub fn sweep_amount(&mut self, base: u16) -> i16 {
        let mut sweep = (base >> self.shift) as i16;
        if self.negate {
            if self.complement == Complement::One {
                return -sweep - 1;
            } else {
                return -sweep;
            }
        }

        self.last_change = sweep as u16;
        sweep
    }
}


#[cfg(test)]
mod tests {
    use super::*;



}