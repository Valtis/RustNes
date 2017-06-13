extern crate sdl2;
use self::sdl2::audio::{AudioQueue};

mod pulse_channel;
mod triangle_channel;
mod envelope;
mod length_counter;
mod timer;

use memory::Memory;

use self::pulse_channel::{PulseChannel, Complement};
use self::triangle_channel::TriangleChannel;
use self::envelope::Envelope;

use std::collections::VecDeque;



const APU_STATUS_REGISTER : u16 = 0x4015;
const FRAME_COUNTER_REGISTER : u16 = 0x4017;
/*

TODO:
    * Implement triangle wave, noise, dmc channels
    * Implement SDL backend
    * Verify that register writes into pulse channels do the right things
        * Write into 0x4003/0x4007 should reset phase
*/

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
    cycle: u16,
    interrupt_disabled: bool,
    interrupt_flag: bool,
    even_cycle: bool
}

impl FrameCounter {
    fn new() -> FrameCounter {
         FrameCounter {
            mode: FrameMode::Mode0,
            cycle: 0,
            interrupt_disabled: true,
            interrupt_flag: false,
            even_cycle: false
        }
    }

    fn cycle(&mut self) -> CycleState {
        self.cycle += 1;
        // apu cycles at uneven cycles (like at cycle 3728.5)
        // the mod ensures that the apu remains in sync
        let cycle_mod = if self.even_cycle {
            1
        } else {
            0
        };

        match self.mode {
            FrameMode::Mode0 => {
                if self.cycle == 0 {
                    self.interrupt();
                } else if (self.cycle == 3728 + cycle_mod) ||
                    (self.cycle == 11185 + cycle_mod) {
                    return CycleState::QuarterFrameCycle;
                } else if self.cycle == 7456 + cycle_mod {
                    return CycleState::HalfFrameCycle;
                } else if self.cycle == 14914 {
                    self.interrupt();
                }

                if self.cycle == 14914 + cycle_mod {
                    self.cycle = 0;
                    self.even_cycle = !self.even_cycle;
                    self.interrupt();
                    return CycleState::HalfFrameCycle;
                }

            },
            FrameMode::Mode1 => {
                if self.cycle == 0 {
                    self.interrupt();
                } else if (self.cycle == 3728 + cycle_mod) ||
                    (self.cycle == 11185 + cycle_mod) {
                    return CycleState::QuarterFrameCycle;
                } else if self.cycle == 7456 + cycle_mod {
                    return CycleState::HalfFrameCycle;
                } else if self.cycle == 14914 {
                    self.interrupt();
                }

                if self.cycle == 18640 + cycle_mod {
                    self.cycle = 0;
                    self.even_cycle = !self.even_cycle;
                    self.interrupt();
                    return CycleState::HalfFrameCycle;
                }

            }
        }

        return CycleState::NormalCycle
    }

    fn interrupt(&mut self) {
        if !self.interrupt_disabled {
            self.interrupt_flag = true;
        } else {

        }
    }

    fn clear_interrupt(&mut self) {
        self.interrupt_flag = false;
    }
}

pub struct Apu {
    pulse_channel_1: PulseChannel,
    pulse_channel_2: PulseChannel,
    triangle_channel: TriangleChannel,
    frame_counter: FrameCounter,
    buffer: Vec<f32>,
    sample_cycle: usize,
    cycles_per_sample: usize,
    max_samples_before_clearing_buffer: usize,
    audio_queue: AudioQueue<f32>
}

impl Memory for Apu {
    fn read(&mut self,  address: u16) -> u8 {
        if address == APU_STATUS_REGISTER {
            self.frame_counter.clear_interrupt();
            0 // TODO: Implement
        } else {
            panic!("APU register reads not implemented yet");
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
            //println!("Noise channel unimplemented");
        } else if address >= 0x4010 && address <= 0x4013 {
            //println!("DMC channel unimplemented");
        } else if address == APU_STATUS_REGISTER {
            let enable_dmc = (0b0001_0000 & value) != 0;
            let enable_noise = (0b0000_1000 & value) != 0;
            let enable_triangle = (0b0000_0100 & value) != 0;
            let enable_pulse_2 = (0b0000_0010 & value) != 0;
            let enable_pulse_1 = (0b0000_0001 & value) != 0;

            self.pulse_channel_1.enable_channel(enable_pulse_1);
            self.pulse_channel_2.enable_channel(enable_pulse_2);
            self.triangle_channel.enable_channel(enable_triangle);
            // TODO: Remaining channels
        } else if address == FRAME_COUNTER_REGISTER {
            let mode = (0b1000_0000 & value) != 0;
            let interrupt_disable_flag = (0b0100_0000 & value) != 0;

            self.frame_counter.cycle = 0;
            if mode {
                self.frame_counter.mode = FrameMode::Mode1;
            } else {
                self.frame_counter.mode = FrameMode::Mode0;
            }

            self.frame_counter.interrupt_disabled = interrupt_disable_flag;

            if interrupt_disable_flag {
                self.frame_counter.interrupt_flag = false;
            }

        } else {
            panic!("Invalid write address {:0x} for APU: ", address);
        }
    }
}

impl Apu {
    pub fn new(audio_queue: AudioQueue<f32>) -> Apu {
        Apu {
            pulse_channel_1: PulseChannel::new(Complement::One),
            pulse_channel_2: PulseChannel::new(Complement::Two),
            triangle_channel: TriangleChannel::new(),
            frame_counter: FrameCounter::new(),
            buffer: vec![],
            sample_cycle: 0,
            cycles_per_sample: 0,
            max_samples_before_clearing_buffer: 0,
            audio_queue: audio_queue,
        }
    }

    pub fn samples(&mut self, samples: u16) {
        self.buffer.resize(samples as usize, 0.0);
        self.max_samples_before_clearing_buffer = samples as usize;
    }

    pub fn set_sampling_rate(&mut self, cpu_frequency: f64, sample_rate: i32) {
        let cpu_cycles_per_apu_cycle = 2.0;
        self.cycles_per_sample =
            ((cpu_frequency*1000_000.0 / cpu_cycles_per_apu_cycle)
            / sample_rate as f64) as usize;
    }

    pub fn execute_cycle(&mut self) {
        self.emulate_channels();
        self.gather_sample();
    }

    fn emulate_channels(&mut self) {
        match self.frame_counter.cycle() {
            CycleState::QuarterFrameCycle => {
                self.pulse_channel_1.cycle_envelope();
                self.pulse_channel_2.cycle_envelope();

                self.triangle_channel.cycle_linear_counter();

            },
            CycleState::HalfFrameCycle => {
                self.pulse_channel_1.cycle_envelope();
                self.pulse_channel_2.cycle_envelope();

                self.pulse_channel_1.cycle_length_counter();
                self.pulse_channel_2.cycle_length_counter();

                self.pulse_channel_1.cycle_sweep_unit();
                self.pulse_channel_2.cycle_sweep_unit();

                self.triangle_channel.cycle_length_counter();

            },
            CycleState::NormalCycle => {}
        }

        self.pulse_channel_1.cycle_timer();
        self.pulse_channel_2.cycle_timer();

        // triangle channel timer cycles twice for each apu cycle
        self.triangle_channel.cycle_timer();
        self.triangle_channel.cycle_timer();
    }

    fn gather_sample(&mut self) {
        self.sample_cycle += 1;
        // get samples every ~ (apu cycle) / (sample rate) / 2
        // (apu cycle -> 2 cpu cycles)
        if self.sample_cycle >= self.cycles_per_sample {
            let output = self.output() as f32;
            self.buffer.push(output);
          //  self.append_buf(output);
            self.sample_cycle = 0;

            if self.buffer.len() >= self.max_samples_before_clearing_buffer {
                self.audio_queue.queue(self.buffer.as_slice());
                self.buffer.clear();
            }
        }
    }

    pub fn pending_interrupt(&self) -> bool {
        self.frame_counter.interrupt_flag && !self.frame_counter.interrupt_disabled
    }

    fn output(&self) -> f64 {
        let pulse_output =
            0.00752*(
                self.pulse_channel_1.output() +
                self.pulse_channel_2.output());

        let noise_output = 0.0;
        let dmc_output = 0.0;

        let tnd_output = 0.00851 * self.triangle_channel.output() + 0.00494 * noise_output + 0.00335 * dmc_output;
        pulse_output + tnd_output
    }
}