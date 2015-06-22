use rom_reader::Rom;
use rom_reader::TvSystem;

pub fn get_cpu(rom: Rom) -> Cpu {

    Cpu::new(rom.header.tv_system)
}


#[derive(Debug)]
pub struct Cpu {
    color_subcarrier_frequency: f64,
    master_clock_frequency: f64,
    clock_divisor: u8,
    cpu_clock_frequency: f64
}

impl Cpu {
    fn new(tv_system: TvSystem) -> Cpu {

        let mut divisor = 0;
        let mut color_freq = 0.0;
        let mut master_freq = 0.0;

        match tv_system {
            TvSystem::Uninitialized => panic!("Uninitialized tv system type when initializing cpu"),
            TvSystem::PAL => {
                divisor = 16;
                color_freq = 4433618.75 / 1000_000.0;
            },
            TvSystem::NTSC => {
                divisor = 12;
                color_freq = 39375000.0/11.0 / 1000_000.0;
            }
        }

        master_freq = 6.0*color_freq;

        Cpu {
            color_subcarrier_frequency: color_freq,
            master_clock_frequency: master_freq,
            clock_divisor: divisor,
            cpu_clock_frequency: master_freq / divisor as f64
        }
    }
}
