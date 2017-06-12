extern crate sdl2;

use self::sdl2::audio::AudioCallback;

use super::Apu;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Mixer {
    apu: Arc<Mutex<Apu>>,
}

impl Mixer {
    pub fn new(apu: Arc<Mutex<Apu>>) -> Mixer {

        Mixer {
            apu: apu,
        }
    }

}



impl AudioCallback for Mixer {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let buf =
            self
            .apu
            .lock()
            .unwrap_or_else(
                |e| panic!("Unexpected failure when locking APU for sampling: {}"))
            .write_buf(out);
    }
}
