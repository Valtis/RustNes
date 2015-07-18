extern crate sdl2;

use memory::Memory;
use self::sdl2::keyboard::Keycode;

use std::collections::HashMap;

#[derive(Debug)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

#[derive(Debug)]
pub struct Controller {
    controls: HashMap<Keycode, Button>,
    buttons: u8,
    shift: u8,
    strobe: bool,
}


impl Memory for Controller {
    // TODO: Upper 3 bits should maintain the value that the bus had previously; 
    // this is currently not implemented. At least one game uses this (paperboy)
    fn read(&mut self, address: u16) -> u8 {
        assert!(address == 0x4016 || address == 0x4017);
        
        let return_value = ((self.buttons << self.shift) & 0x80) >> 7;
        if self.strobe || self.shift == 7 {
            self.shift = 0;
        } else {
            self.shift += 1;
        }
        return_value
    }

    fn write(&mut self, address: u16, value: u8) {
        assert!(address == 0x4016);
        self.strobe = (value & 0x01) == 0x01;
    }
}

impl Controller {
    pub fn new(optional_controls: Option<HashMap<Keycode, Button>>) -> Controller {
        let controls = match optional_controls {
            Some(x) => x,
            None => {
                let mut defaults = HashMap::new();
                defaults.insert(Keycode::Up, Button::Up);
                defaults.insert(Keycode::Down, Button::Down);
                defaults.insert(Keycode::Left, Button::Left);
                defaults.insert(Keycode::Right, Button::Right);
                defaults.insert(Keycode::Tab, Button::Select);
                defaults.insert(Keycode::Return, Button::Start);
                defaults.insert(Keycode::LCtrl, Button::A);
                defaults.insert(Keycode::LShift, Button::B);

                defaults
            }
        };

        Controller {
            controls: controls,
            shift: 0,
            strobe: false,
            buttons: 0,
        }
    }

    pub fn key_down(&mut self, code: Keycode) {
        if !self.controls.contains_key(&code) {
            return;
        }
        self.buttons = self.buttons | match self.controls[&code] {
            Button::A => 0x80, // set bit 7
            Button::B => 0x40, // set bit 6
            Button::Select => 0x20, // set bit 5
            Button::Start => 0x10, // set bit 4
            Button::Up => 0x08, // set bit 3
            Button::Down => 0x04, // set bit 2
            Button::Left => 0x02,
            Button::Right => 0x01,
        }
    }

    pub fn key_up(&mut self, code: Keycode) {        
        if !self.controls.contains_key(&code) {
            return;
        }
        
        self.buttons = self.buttons & match self.controls[&code] {
            Button::A => 0x7F, // clear bit 7
            Button::B => 0xBF,  // clear bit 6
            Button::Select => 0xDF, // clear bit 5
            Button::Start => 0xEF, // clear bit 4
            Button::Up => 0xF7, // clear bit 3
            Button::Down => 0xFB,
            Button::Left => 0xFD,
            Button::Right => 0xFE,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;
    use controller::sdl2::keyboard::Keycode;
    use std::collections::HashMap;

    fn create_test_controller() -> Controller {
        // independent from defaults so that changes to defaults do not invalidate tests
        let mut test_controls = HashMap::new();
        test_controls.insert(Keycode::Up, Button::Up);
        test_controls.insert(Keycode::Down, Button::Down);
        test_controls.insert(Keycode::Left, Button::Left);
        test_controls.insert(Keycode::Right, Button::Right);
        test_controls.insert(Keycode::Tab, Button::Select);
        test_controls.insert(Keycode::Return, Button::Start);
        test_controls.insert(Keycode::LCtrl, Button::A);
        test_controls.insert(Keycode::LShift, Button::B);

        Controller::new(Some(test_controls))
    }

    #[test]
    #[should_panic]
    fn controller_panics_if_write_is_not_to_0x4016() {
        let mut controller = create_test_controller();
        controller.write(0x4000, 51);
    }

    #[test]
    fn write_to_0x4016_sets_strobe_if_bit_0_is_set() {
        let mut controller = create_test_controller();
        controller.write(0x4016, 0x01);
        assert_eq!(true, controller.strobe)
    }

    #[test]
    fn write_to_0x4016_clears_strobe_if_bit_0_is_clear() {
        let mut controller = create_test_controller();
        controller.strobe = true;
        controller.write(0x4016, 0x00);
        assert_eq!(false, controller.strobe)
    }

    #[test]
    fn read_from_0x4016_keeps_shift_at_0_if_strobe_is_high() {
        let mut controller = create_test_controller();
        controller.strobe = true;
        controller.read(0x4016);
        assert_eq!(0, controller.shift);
    }

    #[test]
    fn read_from_0x4017_keeps_shift_at_0_if_strobe_is_high() {
        let mut controller = create_test_controller();
        controller.strobe = true;
        controller.read(0x4017);
        assert_eq!(0, controller.shift);
    }


    #[test]
    fn read_from_0x4016_increases_shift_if_strobe_is_low() {
        let mut controller = create_test_controller();
        controller.strobe = false;
        controller.read(0x4016);
        assert_eq!(1, controller.shift);
    }

    #[test]
    fn read_from_0x4017_increases_shift_if_strobe_is_low() {
        let mut controller = create_test_controller();
        controller.strobe = false;
        controller.read(0x4017);
        assert_eq!(1, controller.shift);
    }

    #[test]
    fn shift_wraps_around_after_7() {
        let mut controller = create_test_controller();
        controller.strobe = false;
        controller.shift = 7;
        controller.read(0x4017);
        assert_eq!(0, controller.shift);
    }

    #[test]
    fn button_a_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::LCtrl);
        assert_eq!(0x80, controller.buttons & 0x80);
    }

    #[test]
    fn button_a_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x80;
        controller.key_up(Keycode::LCtrl);
        assert_eq!(0x00, controller.buttons & 0x80);
    }

    #[test]
    fn button_b_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::LShift);
        assert_eq!(0x40, controller.buttons & 0x40);
    }

    #[test]
    fn button_b_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x40;
        controller.key_up(Keycode::LShift);
        assert_eq!(0x00, controller.buttons & 0x40);
    }
    
    #[test]
    fn button_select_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Tab);
        assert_eq!(0x20, controller.buttons & 0x20);
    }

    #[test]
    fn button_select_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x20;
        controller.key_up(Keycode::Tab);
        assert_eq!(0x00, controller.buttons & 0x20);
    }
            
    #[test]
    fn button_start_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Return);
        assert_eq!(0x10, controller.buttons & 0x10);
    }

    #[test]
    fn button_start_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x10;
        controller.key_up(Keycode::Return);
        assert_eq!(0x00, controller.buttons & 0x10);
    }
    
    #[test]
    fn button_up_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Up);
        assert_eq!(0x08, controller.buttons & 0x08);
    }

    #[test]
    fn button_up_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x08;
        controller.key_up(Keycode::Up);
        assert_eq!(0x00, controller.buttons & 0x08);
    }    
    
    #[test]
    fn button_down_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Down);
        assert_eq!(0x04, controller.buttons & 0x04);
    }

    #[test]
    fn button_down_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x04;
        controller.key_up(Keycode::Down);
        assert_eq!(0x00, controller.buttons & 0x04);
    }
    
    #[test]
    fn button_left_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Left);
        assert_eq!(0x02, controller.buttons & 0x02);
    }

    #[test]
    fn button_left_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x02;
        controller.key_up(Keycode::Left);
        assert_eq!(0x00, controller.buttons & 0x02);
    }    
    
    #[test]
    fn button_right_bit_is_set_if_key_down_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Right);
        assert_eq!(0x01, controller.buttons & 0x01);
    }

    #[test]
    fn button_right_bit_is_cleared_if_key_up_is_called_with_correct_keycode() {
        let mut controller = create_test_controller();
        controller.buttons = 0x01;
        controller.key_up(Keycode::Right);
        assert_eq!(0x00, controller.buttons & 0x01);
    }
    
    #[test]
    fn a_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::LCtrl);
        assert_eq!(0x01, controller.read(0x4016));
    }
    
    #[test]
    fn b_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::LShift);
        for _ in 0..1 {
            controller.read(0x4016);
        }
        assert_eq!(0x01, controller.read(0x4016));
    }
    
    #[test]
    fn select_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Tab);
        for _ in 0..2 {
            controller.read(0x4016);
        }
        assert_eq!(0x01, controller.read(0x4016));
    }
    
    #[test]
    fn start_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Return);
        for _ in 0..3 {
            controller.read(0x4016);
        }
        assert_eq!(0x01, controller.read(0x4016));
    }
    
    #[test]
    fn up_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Up);
        for _ in 0..4 {
            controller.read(0x4016);
        }
        assert_eq!(0x01, controller.read(0x4016));
    }    
    
    #[test]
    fn down_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Down);
        for _ in 0..5 {
            controller.read(0x4016);
        }
        assert_eq!(0x01, controller.read(0x4016));
    }
    
    #[test]
    fn left_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Left);
        for _ in 0..6 {
            controller.read(0x4016);
        }
        assert_eq!(0x01, controller.read(0x4016));
    }
    
    #[test]
    fn right_button_status_is_correctly_returned_when_reading_from_0x4016() {
        let mut controller = create_test_controller();
        controller.key_down(Keycode::Right);
        for _ in 0..7 {
            controller.read(0x4016);
        }
        assert_eq!(0x01, controller.read(0x4016));
    }
}
