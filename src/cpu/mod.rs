use rom::TvSystem;


pub fn get_cpu(tv_system: &TvSystem) -> Cpu {
    Cpu::new(tv_system)
}


#[derive(Debug)]
pub struct Cpu {
    pub frequency: Frequency,
    pub program_counter:u16,
    pub stack_pointer:u8,
    pub wait_counter: u8, // used by instructions that take more than 1 cycle to complete
    pub status_flags:u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
}

impl Cpu {
    fn new(tv_system: &TvSystem) -> Cpu {
        Cpu {
            frequency: Frequency::new(&tv_system),
            program_counter: 0,
            stack_pointer: $FD,
            status_flags: 0x34, // unused 4 and 5 bits to 1; interrupt flag at 2 bit to 1
            wait_counter: 0,
            a: 0,
            x: 0,
            y: 0,
        }
    }
}
#[derive(Debug)]
pub struct Frequency {
    color_subcarrier_frequency: f64,
    master_clock_frequency: f64,
    clock_divisor: u8,
    pub cpu_clock_frequency: f64
}


impl Frequency {
    fn new(tv_system: &TvSystem) -> Frequency {

        let mut divisor:u8;
        let mut color_freq:f64;
        let mut master_freq:f64;

        match *tv_system {
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

        Frequency {
            color_subcarrier_frequency: color_freq,
            master_clock_frequency: master_freq,
            clock_divisor: divisor,
            cpu_clock_frequency: master_freq / divisor as f64
        }
    }
}
