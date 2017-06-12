
pub struct Divider {
	length: u8,
	counter: u8,
}


impl Divider {
	fn new() -> Divider {
		Divider {
			length: 0,
			counter: 0,
		}
	}
}


// in constant volume mode, output is Divider length minus one, otherwise the volume is the counter value
pub struct Envelope {
	start_flag: bool,
	loop_flag: bool,
	constant_volume: bool,
	divider: Divider,
	counter: u8,
}

impl Envelope {
	pub fn new() -> Envelope {
		Envelope {
			start_flag: false,
			loop_flag: false,
			constant_volume: false,
			divider: Divider::new(),
			counter: 15,
		}
	}

	pub fn cycle(&mut self) {
		if self.start_flag {
			self.start_flag = false;
			self.counter = 15;
			self.divider.counter = self.divider.length;
		} else {
			if self.divider.counter == 0 {
				self.divider.counter = self.divider.length;
				if self.counter == 0 {
					if self.loop_flag {
						self.counter = 15;
					}
				} else {
					self.counter -= 1;
				}
			} else {
				self.divider.counter -= 1;
			}
		}
	}

	pub fn set_constant_volume(&mut self, constant: bool) {
		self.constant_volume = constant;
	}

	pub fn set_constant_volume_or_envelope_period(&mut self, value: u8) {
		self.divider.length = value + 1;
	}

	pub fn restart_envelope(&mut self) {
		self.start_flag = true;
	}

	pub fn volume(&self) -> u8 {
		if self.constant_volume {
			self.divider.length
		} else {
			self.counter
		}
	}
}



#[cfg(test)]
mod tests {
    use super::*;


	fn create_test_envelope() -> Envelope {
		Envelope::new()
	}


	#[test]
	fn start_flag_is_cleared_if_start_flag_is_set_when_executing_cycle() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = true;
		envelope.cycle();
		assert_eq!(false, envelope.start_flag);
	}

	#[test]
	fn divider_period_is_reloaded_if_start_flag_is_set_when_executing_cycle() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = true;
		envelope.divider.length = 20;
		envelope.divider.counter = 5;

		envelope.cycle();
		assert_eq!(20, envelope.divider.counter);
	}

	#[test]
	fn divider_period_is_decremented_if_start_flag_is_not_set_and_constant_volume_is_not_set_when_executing_cycle() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.constant_volume = false;
		envelope.divider.length = 20;
		envelope.divider.counter = 5;

		envelope.cycle();
		assert_eq!(4, envelope.divider.counter);
	}

	#[test]
	fn divider_period_is_decremented_if_start_flag_is_not_set_and_constant_volume_is_set_when_executing_cycle() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.constant_volume = true;
		envelope.divider.length = 20;
		envelope.divider.counter = 5;

		envelope.cycle();
		assert_eq!(4, envelope.divider.counter);
	}

	#[test]
	fn divider_period_is_wraps_around_if_start_flag_is_not_set_when_executing_cycle() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.divider.length = 20;
		envelope.divider.counter = 0;

		envelope.cycle();
		assert_eq!(20, envelope.divider.counter);
	}

	#[test]
	fn envelope_counter_is_set_to_15_if_start_flag_is_set() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = true;
		envelope.counter = 10;
		envelope.cycle();
		assert_eq!(15, envelope.counter);
	}

	#[test]
	fn envelope_counter_value_is_not_changed_if_start_flag_is_not_set_and_divider_is_nonzero_before_cycling() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.divider.length = 20;
		envelope.divider.counter = 4;
		envelope.counter = 5;
		envelope.cycle();

		assert_eq!(5, envelope.counter);
	}

	#[test]
	fn envelope_counter_value_is_decremented_if_start_flag_is_not_set_and_constant_volume_is_not_set_divider_is_zero_before_cycling() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.constant_volume = false;
		envelope.divider.length = 20;
		envelope.divider.counter = 0;
		envelope.counter = 5;
		envelope.cycle();

		assert_eq!(4, envelope.counter);
	}

	#[test]
	fn envelope_counter_value_is_decremented_if_start_flag_is_not_set_and_constant_volume_is_set_divider_is_zero_before_cycling() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.constant_volume = true;
		envelope.divider.length = 20;
		envelope.divider.counter = 0;
		envelope.counter = 5;
		envelope.cycle();

		assert_eq!(4, envelope.counter);
	}

	#[test]
	fn envelope_counter_value_remains_zero_if_start_flag_is_not_set_and_loop_flag_is_not_set_and_divider_is_zero_before_cycling() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.loop_flag = false;
		envelope.divider.length = 20;
		envelope.divider.counter = 0;
		envelope.counter = 0;
		envelope.cycle();

		assert_eq!(0, envelope.counter);
	}

	#[test]
	fn envelope_counter_value_is_set_to_15_if_loop_flag_is_set_when_divider_is_zero_and_counter_is_zero_when_cycling() {
		let mut envelope = create_test_envelope();
		envelope.start_flag = false;
		envelope.loop_flag = true;
		envelope.divider.length = 20;
		envelope.divider.counter = 0;
		envelope.counter = 0;
		envelope.cycle();

		assert_eq!(15, envelope.counter);
	}

	#[test]
	fn envelope_volume_is_divider_length_if_constant_flag_is_set() {
		let mut envelope = create_test_envelope();
		envelope.constant_volume = true;
		envelope.divider.length = 20;
		assert_eq!(20, envelope.volume());
	}

	#[test]
	fn envelope_volume_is_not_affected_by_cycling_if_constant_flag_is_set() {
		let mut envelope = create_test_envelope();
		envelope.constant_volume = true;
		envelope.divider.length = 20;
		envelope.cycle();
		assert_eq!(19, envelope.volume());
	}

	#[test]
	fn envelope_volume_is_counter_value_if_constant_volume_flag_is_clear() {
		let mut envelope = create_test_envelope();
		envelope.constant_volume = false;
		envelope.divider.length = 20;
		envelope.counter = 4;
		assert_eq!(4, envelope.volume());
	}
}