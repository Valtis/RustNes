extern crate sdl2;
use self::sdl2::audio::{AudioQueue, AudioFormatNum};

mod pulse_channel;
mod triangle_channel;
mod noise_channel;
mod dmc_channel;
mod envelope;
mod sweep;
mod length_counter;
mod linear_counter;
mod timer;

use memory::Memory;

use self::pulse_channel::{PulseChannel};
use self::sweep::Complement;
use self::triangle_channel::TriangleChannel;
use self::noise_channel::NoiseChannel;
use self::dmc_channel::DmcChannel;
use self::envelope::Envelope;
use self::sweep::Sweep;

use std::collections::VecDeque;
use std::cell::RefCell;
use std::rc::Rc;

const APU_STATUS_REGISTER : u16 = 0x4015;
const FRAME_COUNTER_REGISTER : u16 = 0x4017;

#[derive(Debug)]
enum FrameMode {
    Mode0, // 4 step mode
    Mode1 // 5 step mode
}

enum CycleState {
    QuarterFrameCycle,
    HalfFrameCycle,
    NormalCycle,
}

struct FrameCounter {
    mode: FrameMode,
    cycle: u32,
    interrupt_disabled: bool,
    interrupt_flag: bool,
    reset_timer_flag: bool,
    reset_cycle: u8,
}

impl FrameCounter {
    fn new() -> FrameCounter {
         FrameCounter {
            mode: FrameMode::Mode0,
            cycle: 0,
            interrupt_disabled: true,
            interrupt_flag: false,
            reset_timer_flag: false,
            reset_cycle: 0,
        }
    }

    fn cycle(&mut self) -> CycleState {

        let mut retval = CycleState::NormalCycle;
        if self.reset_timer_flag {
            let cpu_cycles_for_reset = 2;
            if self.reset_cycle < cpu_cycles_for_reset {
                self.reset_cycle += 1;
            } else {
                self.reset_cycle = 0;
                self.reset_timer_flag = false;
                self.cycle = 0;
            }
        }
        // cycle counts below are the documented frame cycles
        // multiplied by two, as this function is called twice for each
        // apu cycle. This fixes the half-cycle issue with timings
        // where some actions occur on half cycle (at apu cycle 3728.5
        // for example)
        match self.mode {
            FrameMode::Mode0 => {
                if self.cycle == 7457 ||
                    self.cycle == 22371 {
                    retval = CycleState::QuarterFrameCycle;
                } else if self.cycle == 14913 {
                    retval = CycleState::HalfFrameCycle;
                } else if self.cycle == 29828 {
                    self.interrupt();
                } else if self.cycle == 29829 {
                    self.interrupt();
                    retval = CycleState::HalfFrameCycle;
                } else if self.cycle == 29830 {
                    self.interrupt();
                    self.cycle = 0;
                }
            },
            FrameMode::Mode1 => {
                if self.cycle == 7457 ||
                    self.cycle == 22371 {
                    retval = CycleState::QuarterFrameCycle;
                } else if self.cycle == 14913 {
                    retval = CycleState::HalfFrameCycle;
                } else if self.cycle == 37281 {
                    self.cycle = 0;
                    retval = CycleState::HalfFrameCycle;
                }
            }
        }

        self.cycle += 1;
        return retval
    }

    fn interrupt(&mut self) {
        if !self.interrupt_disabled {
            self.interrupt_flag = true;
        }
    }

    fn clear_interrupt(&mut self) {
        self.interrupt_flag = false;
    }
}

// for mocking, primarily
pub trait Audio<T : AudioFormatNum> {
    fn queue(&mut self, slice: &[T]);
}

pub struct SDLAudio<T : AudioFormatNum> {
    audio_queue: AudioQueue<T>,
}

impl<T : AudioFormatNum> SDLAudio<T> {
    pub fn new(queue: AudioQueue<T>) -> SDLAudio<T> {
        SDLAudio {
            audio_queue: queue,
        }
    }
}

impl<T : AudioFormatNum> Audio<T> for SDLAudio<T> {
    fn queue(&mut self, slice: &[T]) {
        self.audio_queue.queue(slice);
    }
}


pub struct Apu<'a> {
    pulse_channel_1: PulseChannel,
    pulse_channel_2: PulseChannel,
    triangle_channel: TriangleChannel,
    noise_channel: NoiseChannel,
    dmc_channel: DmcChannel<'a>,
    frame_counter: FrameCounter,
    buffer: Vec<f32>,
    sample_cycle: f64,
    cycles_per_sample: f64,
    max_samples_before_clearing_buffer: usize,
    audio_queue: Box<Audio<f32>>,
    is_even_cycle: bool,
}

impl<'a> Memory for Apu<'a> {
    fn read(&mut self,  address: u16) -> u8 {
        if address == APU_STATUS_REGISTER {

            let mut ret = 0;

            if self.dmc_channel.pending_interrupt() {
                ret |= 0b1000_0000;
            }

            if self.frame_counter.interrupt_flag {
                ret |= 0b0100_0000;
            }


            if self.dmc_channel.active() {
                ret |= 0b0001_0000;
            }

            if self.noise_channel.length_counter_nonzero() {
                ret |= 0b0000_1000;
            }

            if self.triangle_channel.length_counter_nonzero() {
                ret |= 0b0000_0100;
            }

            if self.pulse_channel_2.length_counter_nonzero() {
                ret |= 0b0000_0010;
            }

            if self.pulse_channel_1.length_counter_nonzero() {
                ret |= 0b0000_0001;
            }

            self.frame_counter.clear_interrupt();
            return ret;

        } else {
            panic!("Invalid APU register read for {:0x}", address);
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address >= 0x4000 && address <= 0x4003 {
            self.pulse_channel_1.write(address, value);
        } else if address >= 0x4004 && address <= 0x4007 {
            self.pulse_channel_2.write(address, value);
        } else if address >= 0x4008 && address <= 0x400B {
            self.triangle_channel.write(address, value);
        } else if address >= 0x400C && address <= 0x400F {
            self.noise_channel.write(address, value);
        } else if address >= 0x4010 && address <= 0x4013 {
            self.dmc_channel.write(address, value);
        } else if address == APU_STATUS_REGISTER {

            self.dmc_channel.clear_interrupt();
            let enable_dmc = (0b0001_0000 & value) != 0;
            let enable_noise = (0b0000_1000 & value) != 0;
            let enable_triangle = (0b0000_0100 & value) != 0;
            let enable_pulse_2 = (0b0000_0010 & value) != 0;
            let enable_pulse_1 = (0b0000_0001 & value) != 0;

            self.pulse_channel_1.enable_channel(enable_pulse_1);
            self.pulse_channel_2.enable_channel(enable_pulse_2);
            self.triangle_channel.enable_channel(enable_triangle);
            self.noise_channel.enable_channel(enable_noise);
            self.dmc_channel.enable_channel(enable_dmc);
        } else if address == FRAME_COUNTER_REGISTER {
            let mode = (0b1000_0000 & value) != 0;
            let interrupt_disable_flag = (0b0100_0000 & value) != 0;

            self.frame_counter.cycle = 0;
            if mode {
                self.frame_counter.mode = FrameMode::Mode1;
                self.quarter_frame_cycle();
                self.half_frame_cycle();
            } else {
                self.frame_counter.mode = FrameMode::Mode0;
            }

            self.frame_counter.interrupt_disabled = interrupt_disable_flag;

            if interrupt_disable_flag {
                self.frame_counter.interrupt_flag = false;
            }

            self.frame_counter.reset_timer_flag = true;
        } else {
            panic!("Invalid write address {:0x} for APU: ", address);
        }
    }
}

impl<'a> Apu<'a> {
    pub fn new(audio_queue: Box<Audio<f32>>) -> Apu<'a> {
        Apu {
            pulse_channel_1: PulseChannel::new(Complement::One),
            pulse_channel_2: PulseChannel::new(Complement::Two),
            triangle_channel: TriangleChannel::new(),
            noise_channel: NoiseChannel::new(),
            dmc_channel: DmcChannel::new(),
            frame_counter: FrameCounter::new(),
            buffer: vec![],
            sample_cycle: 0.0,
            cycles_per_sample: 0.0,
            max_samples_before_clearing_buffer: 0,
            audio_queue: audio_queue,
            is_even_cycle: false,
        }
    }

    pub fn samples(&mut self, samples: u16) {
        self.buffer.resize(samples as usize, 0.0);
        self.max_samples_before_clearing_buffer = samples as usize;
    }

    pub fn set_sampling_rate(&mut self, cpu_frequency: f64, sample_rate: i32) {
        self.cycles_per_sample =
            ((cpu_frequency*1000_000.0) / sample_rate as f64);
    }

    // called once for each cpu cycle
    // apu itself cycles once for each cpu cycle,
    // but frame counter cycles are subtly wrong
    // if we call this once for every two cpu cycles
    // frame counter
    pub fn execute_cycle(&mut self) {
        self.emulate_channels();
        self.gather_sample();
    }

    fn emulate_channels(&mut self) {
        match self.frame_counter.cycle() {
            CycleState::QuarterFrameCycle => {
                self.quarter_frame_cycle();
            },
            CycleState::HalfFrameCycle => {
                // a quarter frame cycle is also run during a half frame cycle
                self.quarter_frame_cycle();
                self.half_frame_cycle();
            },
            CycleState::NormalCycle => {}
        }

        if self.is_even_cycle {

            self.pulse_channel_1.cycle_timer();
            self.pulse_channel_2.cycle_timer();

            self.triangle_channel.cycle_timer();

            self.noise_channel.cycle_timer();

            self.dmc_channel.cycle_timer();
        } else {
            // triangle channel timer cycles twice for each apu cycle
            // --> it cycles once for each cpu cycle
            self.triangle_channel.cycle_timer();
            // dmc cycles internally using cpu timings as well
            self.dmc_channel.cycle_timer();
        }

        self.is_even_cycle = !self.is_even_cycle;
    }

    fn quarter_frame_cycle(&mut self) {
        self.pulse_channel_1.cycle_envelope();
        self.pulse_channel_2.cycle_envelope();
        self.noise_channel.cycle_envelope();
        self.triangle_channel.cycle_linear_counter();
    }

    fn half_frame_cycle(&mut self) {
        self.pulse_channel_1.cycle_length_counter();
        self.pulse_channel_2.cycle_length_counter();
        self.triangle_channel.cycle_length_counter();
        self.noise_channel.cycle_length_counter();

        self.pulse_channel_1.cycle_sweep_unit();
        self.pulse_channel_2.cycle_sweep_unit();
    }

    fn gather_sample(&mut self) {
        self.sample_cycle += 1.0;
        // get samples every ~ (apu cycle) / (sample rate) / 2
        // (apu cycle -> 2 cpu cycles)
        if self.sample_cycle >= self.cycles_per_sample {
            let output = self.output() as f32;
            self.buffer.push(output);
            self.sample_cycle -= self.cycles_per_sample;

            if self.buffer.len() >= self.max_samples_before_clearing_buffer {
                self.audio_queue.queue(self.buffer.as_slice());
                self.buffer.clear();
            }
        }
    }

    pub fn pending_interrupt(&self) -> bool {
        self.frame_counter.interrupt_flag ||
        self.dmc_channel.pending_interrupt()
    }

    pub fn set_memory(&mut self, mem: Rc<RefCell<Box<Memory + 'a>>>) {
        self.dmc_channel.set_memory(mem);
    }

    fn output(&self) -> f64 {
        let pulse_output =
            0.00752*(
                self.pulse_channel_1.output() +
                self.pulse_channel_2.output());

        let tnd_output =
            0.00851*self.triangle_channel.output()
            + 0.00494*self.noise_channel.output()
            + 0.00335*self.dmc_channel.output();
        pulse_output + tnd_output
    }

    pub fn delay_cpu(&mut self) -> u8 {
        self.dmc_channel.delay_cpu()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;
    use std::rc::Rc;
    use std::cell::RefCell;

    struct MockAudio {
    }

    impl MockAudio {
        fn new() -> MockAudio {
            MockAudio {

            }
        }
    }

    impl Audio<f32> for MockAudio {
        fn queue(&mut self, slice: &[f32]) {

        }
    }

    struct MockMemory {

    }

    impl MockMemory {
        fn new() -> MockMemory {
            MockMemory {

            }
        }
    }

    impl Memory for MockMemory {
        fn read(&mut self, address: u16) -> u8 {
            0
        }

        fn write(&mut self, address: u16, value: u8) {

        }
    }

    fn create_test_apu<'a>() -> Apu<'a> {

        let audio = Box::new(MockAudio::new());
        let mut apu = Apu::new(audio);

        let mem = Rc::new(
            RefCell::new(
                Box::new(MockMemory::new()) as Box<Memory>));
        apu.set_memory(mem);
        apu
    }

    fn delay_dmc(apu: &mut Apu, count: u16) {
        for _ in 0..apu.dmc_channel.dmc_rate()*8*count {
            apu.execute_cycle();
        }
    }

    // tests below based on test roms by blargg
    #[test]
    fn reading_0x4015_does_not_clear_dmc_interrupt() {
        let mut apu = create_test_apu();

        apu.write(0x4010, 0x8F);
        apu.write(0x4012, 0x100);
        apu.write(0x4013, 1);
        apu.write(APU_STATUS_REGISTER, 0x10);
        assert_eq!(apu.read(0x4015), 0x10);
        delay_dmc(&mut apu, 20);
        assert_eq!(apu.read(0x4015), 0x80);
        // flag should still be there
        assert_eq!(apu.read(0x4015), 0x80);
    }

    #[test]
    fn reading_0x4015_clears_frame_interrupt() {

        let mut apu = create_test_apu();

        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..30000 {
            apu.execute_cycle();
        }

        assert!(apu.frame_counter.interrupt_flag);
        apu.read(APU_STATUS_REGISTER);
        assert!(!apu.frame_counter.interrupt_flag);
    }

    #[test]
    fn writing_to_0x4015_clears_dmc_interrupt_flag() {
        let mut apu = create_test_apu();

        apu.write(0x4010, 0x8F);
        apu.write(0x4012, 0x100);
        apu.write(0x4013, 1);
        apu.write(APU_STATUS_REGISTER, 0x10);
        assert_eq!(apu.read(APU_STATUS_REGISTER), 0x10);
        delay_dmc(&mut apu, 20);
        assert_eq!(apu.read(APU_STATUS_REGISTER), 0x80);
        apu.write(APU_STATUS_REGISTER, 0x10);
        assert_eq!(apu.read(APU_STATUS_REGISTER), 0x10);
    }

    #[test]
    fn pulse_channel_1_length_counter_load_works_and_status_reg_has_value() {
        let mut apu = create_test_apu();
        assert_eq!(apu.read(0x4015) & 0b0000_0001, 0b0000_0000);
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4003, (6 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0001, 0b0000_0001);
    }

    #[test]
    fn pulse_channel_2_length_counter_load_works_and_status_reg_has_value() {
        let mut apu = create_test_apu();
        assert_eq!(apu.read(0x4015) & 0b0000_0010, 0b0000_0000);
        apu.write(0x4015, 0b0000_0010);
        apu.write(0x4007, (6 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn triangle_channel_length_counter_load_works_and_status_reg_has_value() {
        let mut apu = create_test_apu();
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0000);
        apu.write(0x4015, 0b0000_0100);
        apu.write(0x400B, (6 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0100);
    }

    #[test]
    fn noise_channel_length_counter_load_works_and_status_reg_has_value() {
        let mut apu = create_test_apu();
        assert_eq!(apu.read(0x4015) & 0b0000_1000, 0b0000_0000);
        apu.write(0x4015, 0b0000_1000);
        apu.write(0x400F, (6 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_1000, 0b0000_1000);
    }

    #[test]
    fn pulse_channel_1_lenght_counter_ticks_down_to_zero() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4003, (6 & 0b0001_1111) << 3);

        // actual value does not matter, as long as it is longer than the
        // frame counter cycles at least once
        let frame_counter_cycles = 38000;

        // as above, as long as it is larger than max length counter length,
        // we're good
        let length_counter_cycles_per_frame = 2;
        let length_counter_cycles = 200 / length_counter_cycles_per_frame;

        // length counter should have reached zero by the end of the loop
        for _ in 0..frame_counter_cycles*length_counter_cycles {
            apu.execute_cycle();
        }

        assert_eq!(apu.read(0x4015) & 0b0000_0001, 0b0000_0000);
    }

    #[test]
    fn pulse_channel_2_length_counter_ticks_down_to_zero() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0010);
        apu.write(0x4007, (6 & 0b0001_1111) << 3);


        // actual value does not matter, as long as it is longer than the
        // frame counter cycles at least once
        let frame_counter_cycles = 38000;

        // as above, as long as it is larger than max length counter length,
        // we're good
        let length_counter_cycles_per_frame = 2;
        let length_counter_cycles = 200 / length_counter_cycles_per_frame;

        // length counter should have reached zero by the end of the loop
        for _ in 0..frame_counter_cycles*length_counter_cycles {
            apu.execute_cycle();
        }
        assert_eq!(apu.read(0x4015) & 0b0000_0010, 0b0000_0000);
    }

    #[test]
    fn triangle_channel_length_counter_ticks_down_to_zero() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0100);
        apu.write(0x4007, (6 % 0b0001_1111) << 3);

          // frame counter cycles at least once
        let frame_counter_cycles = 38000;

        // as above, as long as it is larger than max length counter length,
        // we're good
        let length_counter_cycles_per_frame = 2;
        let length_counter_cycles = 200 / length_counter_cycles_per_frame;

        // length counter should have reached zero by the end of the loop
        for _ in 0..frame_counter_cycles*length_counter_cycles {
            apu.execute_cycle();
        }
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0000);
    }

    #[test]
    fn triangle_channel_linear_counter_ticks_down_to_zero() {
    let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0100);
        apu.write(0x4007, (6 % 0b0001_1111) << 3);

          // frame counter cycles at least once
        let frame_counter_cycles = 38000;

        // as above, as long as it is larger than max length counter length,
        // we're good
        let length_counter_cycles_per_frame = 2;
        let length_counter_cycles = 200 / length_counter_cycles_per_frame;

        // length counter should have reached zero by the end of the loop
        for _ in 0..frame_counter_cycles*length_counter_cycles {
            apu.execute_cycle();
        }
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0000);
    }

    #[test]
    fn noise_channel_length_counter_ticks_down_to_zero() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_1000);
        apu.write(0x400F, (6 % 0b0001_1111) << 3);

          // frame counter cycles at least once
        let frame_counter_cycles = 38000;

        // as above, as long as it is larger than max length counter length,
        // we're good
        let length_counter_cycles_per_frame = 2;
        let length_counter_cycles = 200 / length_counter_cycles_per_frame;

        // length counter should have reached zero by the end of the loop
        for _ in 0..frame_counter_cycles*length_counter_cycles {
            apu.execute_cycle();
        }
        assert_eq!(apu.read(0x4015) & 0b0000_1000, 0b0000_0000);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_pulse_channel_1_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4003, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0001, 0b0000_0001);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(apu.read(0x4015) & 0b0000_0001, 0b0000_0000);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_pulse_channel_1_sweep_unit() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4001, 0b1111_0101);
        // reload flag is set initially, so first cycle just reloads the value
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(
            apu.pulse_channel_1.sweep.counter,
            apu.pulse_channel_1.sweep.length - 1);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_pulse_channel_1_envelope() {
        let mut apu = create_test_apu();
        apu.write(0x4000, 0b0010_1111);
        // side-effect: sets the envelope start flag
        apu.write(0x4003, 0b1111_1111);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(
            apu.pulse_channel_1.envelope.divider.counter,
            apu.pulse_channel_1.envelope.divider.length - 1);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_pulse_channel_2_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0010);
        apu.write(0x4007, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0010, 0b0000_0010);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(apu.read(0x4015) & 0b0000_0010, 0b0000_0000);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_pulse_channel_2_sweep_unit() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4005, 0b1111_0101);
        // reload flag is set initially, so first cycle just reloads the value
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(
            apu.pulse_channel_2.sweep.counter,
            apu.pulse_channel_2.sweep.length - 1);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_pulse_channel_2_envelope() {

        let mut apu = create_test_apu();
        apu.write(0x4004, 0b0010_1111);
        // side-effect: sets the envelope start flag
        apu.write(0x4007, 0b1111_1111);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(
            apu.pulse_channel_2.envelope.divider.counter,
            apu.pulse_channel_2.envelope.divider.length - 1);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_triange_channel_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0100);
        apu.write(0x400B, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0100);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0000);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_triange_channel_linear_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0100);
        apu.write(0x4008, 0b0111_1111);
        apu.write(0x400B, 0); // set linear counter reload flag
        // value is reloaded on the first write, so write twice
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(
            apu.triangle_channel.linear_counter.counter,
            apu.triangle_channel.linear_counter.length - 1);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_noise_channel_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_1000);
        apu.write(0x400F, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_1000, 0b0000_1000);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(apu.read(0x4015) & 0b0000_1000, 0b0000_0000);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_clock_noise_channel_envelope() {
        let mut apu = create_test_apu();
        apu.write(0x400C, 0b0010_1111);
        // side-effect: sets the envelope start flag
        apu.write(0x400F, 0b1111_1111);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(
            apu.noise_channel.envelope.divider.counter,
            apu.noise_channel.envelope.divider.length - 1);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_pulse_channel_1_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4003, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0001, 0b0000_0001);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(apu.read(0x4015) & 0b0000_0001, 0b0000_0001);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_pulse_channel_1_sweep_unit() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4001, 0b1111_0101);
        // clock once so that sweep unit is reloaded
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(
            apu.pulse_channel_1.sweep.counter,
            apu.pulse_channel_1.sweep.length);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_pulse_channeL_1_envelope() {
            let mut apu = create_test_apu();
        apu.write(0x400C, 0b0010_1111);
        // side-effect: sets the envelope start flag
        apu.write(0x400F, 0b1111_1111);
        apu.write(FRAME_COUNTER_REGISTER, 0x80); // reload values
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(
            apu.noise_channel.envelope.divider.counter,
            apu.noise_channel.envelope.divider.length);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_pulse_channel_2_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0010);
        apu.write(0x4007, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0010, 0b0000_0010);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(apu.read(0x4015) & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_pulse_channel_2_sweep_unit() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0001);
        apu.write(0x4005, 0b1111_0101);
        // clock once so that sweep unit is reloaded
        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(
            apu.pulse_channel_2.sweep.counter,
            apu.pulse_channel_2.sweep.length);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_pulse_channeL_2_envelope() {

        let mut apu = create_test_apu();
        apu.write(0x4004, 0b0010_1111);
        // side-effect: sets the envelope start flag
        apu.write(0x4007, 0b1111_1111);
        apu.write(FRAME_COUNTER_REGISTER, 0x80); // reload values
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(
            apu.pulse_channel_2.envelope.divider.counter,
            apu.pulse_channel_2.envelope.divider.length);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_triangle_channel_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0100);
        apu.write(0x400B, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0100);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(apu.read(0x4015) & 0b0000_0100, 0b0000_0100);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_triangle_channel_linear_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_0100);
        apu.write(0x4008, 0b0111_1111);
        apu.write(0x400B, 0); // set linear counter reload flag
        apu.write(FRAME_COUNTER_REGISTER, 0x80); // reloads linear counter
        apu.write(FRAME_COUNTER_REGISTER, 0x00); // should now remain unchanged
        assert_eq!(
            apu.triangle_channel.linear_counter.counter,
            apu.triangle_channel.linear_counter.length);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_noise_channel_length_counter() {
        let mut apu = create_test_apu();
        apu.write(0x4015, 0b0000_1000);
        apu.write(0x400F, (3 & 0b0001_1111) << 3);
        assert_eq!(apu.read(0x4015) & 0b0000_1000, 0b0000_1000);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(apu.read(0x4015) & 0b0000_1000, 0b0000_1000);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_clock_noise_channel_envelope() {
        let mut apu = create_test_apu();
        apu.write(0x400C, 0b0010_1111);
        // side-effect: sets the envelope start flag
        apu.write(0x400F, 0b1111_1111);
        apu.write(FRAME_COUNTER_REGISTER, 0x80); // reload values first
        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(
            apu.noise_channel.envelope.divider.counter,
            apu.noise_channel.envelope.divider.length);
    }

    #[test]
    fn frame_interrupt_flag_should_not_be_set_when_0x40_is_written_into_0x4017() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x40);
        for _ in 0..38000 {
            apu.execute_cycle();
        }

        assert!(!apu.pending_interrupt());
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0);
    }

    #[test]
    fn frame_interrupt_flag_should_not_be_set_when_0x80_is_written_into_0x4017() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x40);
        for _ in 0..38000 {
            apu.execute_cycle();
        }

        assert!(!apu.pending_interrupt());
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0);
    }

    #[test]
    fn frame_interrupt_flag_should_be_set_when_0x00_is_written_into_0x4017() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..38000 {
            apu.execute_cycle();
        }

        assert!(apu.pending_interrupt());
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0x40);
    }

    #[test]
    fn writing_0x00_into_0x4017_should_not_affect_frame_interrupt_flag() {
       let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..38000 {
            apu.execute_cycle();
        }

        apu.write(FRAME_COUNTER_REGISTER, 0x00);
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0x40);
    }

    #[test]
    fn writing_0x80_into_0x4017_should_not_affect_frame_interrupt_flag() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..38000 {
            apu.execute_cycle();
        }

        apu.write(FRAME_COUNTER_REGISTER, 0x80);
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0x40);
    }

    #[test]
    fn writing_0x40_into_0x4017_should_clear_frame_interrupt() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..38000 {
            apu.execute_cycle();
        }

        apu.write(FRAME_COUNTER_REGISTER, 0x40);
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0);
    }

    #[test]
    fn writing_0xC0_into_0x4017_should_clear_frame_interrupt() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..38000 {
            apu.execute_cycle();
        }

        apu.write(FRAME_COUNTER_REGISTER, 0xC0);
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0);
    }

    #[test]
    fn frame_irq_flag_should_be_clear_when_executing_29830_cycles() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..29830 {
            apu.execute_cycle();
        }

        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0);
    }

    #[test]
    fn frame_irq_flag_should_be_set_when_executing_29831_cycles() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..29831 {
            apu.execute_cycle();
        }

        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0x40);
    }

    #[test]
    fn frame_irq_flag_should_be_set_again_at_29832() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..29832 {
            apu.execute_cycle();
        }

        apu.read(APU_STATUS_REGISTER); // resets apu flag
        apu.execute_cycle();
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0x40);
    }

    #[test]
    fn frame_irq_flag_should_not_be_set_again_at_29833() {
        let mut apu = create_test_apu();
        apu.write(FRAME_COUNTER_REGISTER, 0x00);

        for _ in 0..29833 {
            apu.execute_cycle();
        }

        apu.read(APU_STATUS_REGISTER); // resets apu flag
        apu.execute_cycle();
        assert_eq!(apu.read(APU_STATUS_REGISTER) & 0x40, 0);
    }


    #[test]
    fn even_jitter_is_handled_correctly() {

    }
}

