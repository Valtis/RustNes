extern crate time;

use memory::Memory;
use cpu::Cpu;
use rom::read_rom;

use std::cell::RefCell;

pub struct Console{
    cpu: Cpu,
    memory: RefCell<Memory>,
}

impl Console {
    pub fn new (rom_path: &str) -> Console {

        let mut rom = read_rom(rom_path);
        let mut memory = RefCell::new(Memory::new());
        let mut console = Console {
            memory: memory.clone(),
            cpu: Cpu::new(&rom.header.tv_system, memory.clone()),
        };

        console.memory.set_rom(rom);
        console
    }


    pub fn execute(&mut self) {
        let mut avg_cycle = 0.0;
        let cycle_time_in_nanoseconds = (1.0/(self.cpu.frequency.cpu_clock_frequency/1000.0)) as u64;
        println!("CPU frequency: {}", self.cpu.frequency.cpu_clock_frequency);
        println!("Cycle time in nanoseconds: {}", cycle_time_in_nanoseconds);

        let mut cycles = 0;
        let max_cycles = (self.cpu.frequency.cpu_clock_frequency*1000_000.0) as u32;
        let cpu_cycles_per_frame = 100;

        self.cpu.reset();

        // FOR NES CPU TEST
        self.cpu.program_counter = 0xC000;

        println!("\nPC: {}\n", self.cpu.program_counter);

        let mut time = time::precise_time_ns();
        loop {
            let current_time = time::precise_time_ns();
            let time_taken = current_time - time;

            // execute cpu_cycles_per_frame cycles every cpu_cycle_per_frame * cycle_time nanoseconds.
            // the 6502 has frequency around ~2 MHZ whics means that a cycle needs to be
            // executed every ~500ns. This however is not really possible even with high precision
            // timers. However, executing, say, 10 cycles every 5000ns is far more achievable.

            if time_taken > cycle_time_in_nanoseconds * cpu_cycles_per_frame {
                avg_cycle += time_taken as f64;
                for _ in 0..cpu_cycles_per_frame {
                    // ensure instruction timing
                    if self.cpu.wait_counter > 0 {
                        self.cpu.wait_counter -= 1;
                    } else {
                        self.cpu.execute_instruction();
                    }
                    cycles += 1;
                }

                time = current_time;
            }
            if cycles >= max_cycles {
                break;
            }
        }

        println!("Avg cycle length: {}", avg_cycle/max_cycles as f64);
        println!("Duration: {}", avg_cycle as f64/ 1000_000_000.0)
    }
}
