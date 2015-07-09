extern crate time;

use memory::Memory;
use memory_bus::*;
use cpu::Cpu;
use ppu::Ppu;
use rom::read_rom;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Console{
    cpu: Cpu,
    memory: Rc<RefCell<Box<Memory>>>,
    ppu: Rc<RefCell<Ppu>>,
}

impl Console {
    pub fn new (rom_path: &str) -> Console {
        let rom = Box::new(read_rom(rom_path));
        let tv_system = rom.header.tv_system.clone();
        let ppu = Rc::new(RefCell::new(Ppu::new(&tv_system)));

        let mem = Rc::new(RefCell::new(Box::new(MemoryBus::new(rom, ppu.clone())) as Box<Memory>));
        Console {
            memory: mem.clone(),
            cpu: Cpu::new(&tv_system, mem.clone()),
            ppu: ppu.clone(),
        }
    }


    pub fn execute(&mut self) {
        let cpu_cycle_time_in_nanoseconds = (1.0/(self.cpu.frequency.cpu_clock_frequency/1000.0)) as u64;
        println!("CPU frequency: {}", self.cpu.frequency.cpu_clock_frequency);
        println!("Cycle time in nanoseconds: {}", cpu_cycle_time_in_nanoseconds);

        // execute cpu_cycles_per_tick cycles every cpu_cycles_per_tick * tick_time nanoseconds.
        // the 6502 frequency is around ~2 MHZ whics means that a cycle needs to be
        // executed every ~500ns. This however is not really possible even with high precision
        // timers. At least on my computer, best precision I got from timer was 700ns which means
        // there would be ~40% error. Thus, instead of executing one cpu cycle every ~500ns
        // it is better to execute n cycles every n*500ns as this reduces timer errors.

        let cpu_cycles_per_tick = 10;
        // PAL PPU executes exactly 3.2 cycles for each CPU cycle (vs exactly 3 cycles NTSC).
        // this means we need extra cycle every now an then when emulating PAL to maintaing timing
        //let mut ppu_fractional_cycles = 0.0;
        self.cpu.reset();

        let mut time = time::precise_time_ns();
        loop {
            let current_time = time::precise_time_ns();
            let time_taken = current_time - time;


            if time_taken > cpu_cycle_time_in_nanoseconds * cpu_cycles_per_tick {
                for _ in 0..cpu_cycles_per_tick {
                    self.run_emulation_tick();
                }
                time = current_time;
            }

        }
    }

    fn run_emulation_tick(&mut self) {
        // ensure instruction timing
        if self.cpu.wait_counter > 0 {
            self.cpu.wait_counter -= 1;
        } else {
            // check for nmi from ppu
            let nmi_occured = self.ppu.borrow_mut().nmi_occured();
            if nmi_occured {
                self.cpu.handle_nmi();
                return;
            } else {
                self.cpu.execute_instruction();
            }
        }
        // emulate PPU cycles. Executes 3 cycles (NTSC) or average 3.2 cycles (PAL) per cpu cycle.
        // PAL executes 3 cycles with an additional cycle every few cpu cycles to remain in sync
        self.ppu.borrow_mut().execute_cycles();
    }

}
