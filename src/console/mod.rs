extern crate time;
extern crate sdl2;
use self::sdl2::keyboard::Keycode;
use self::sdl2::event::Event;

use memory::Memory;
use memory_bus::*;
use cpu::Cpu;
use ppu::Ppu;
use rom::read_rom;
use ppu::renderer::*;

use std::rc::Rc;
use std::cell::RefCell;



#[derive(Debug)]
pub struct Console {
    cpu: Cpu,
    memory: Rc<RefCell<Box<Memory>>>,
    ppu: Rc<RefCell<Ppu>>,
}

impl Console {

    pub fn execute(rom_path: &str) {

        /*
            Somewhat ugly hack.

            The problem here is that sdl_context must outlive the window and renderer; otherwise when
            the context is destroyed, renderer no longer functions (window is closed) and any attempt to
            render image will silently fail (annoying...). (Oddly enough, the sdl2 library does not enforce
            the lifetime requirements through ownership and it's possible to have live renderer
            object while sdl_context has already been destroyed).

            Sdl_context itself is non-movable\-copyable as there are several SDL subsystems
            that are not thread safe and as such only main thread should hold the object. However as the
            sdl struct is private so we can't even pass it around as reference (let foo: sdl2::sdl::Sdl
            and let foo: &sdl2::sdl::Sdl both cause a compile error. Why type inference seems to work is
            a mystery to me at this point).

            This means sdl_context must be instantiated in main and it can't be stored\passed around.
            The required structs that depend on sdl_context are initialized here as well.
            Ideally I'd do the initialization elsewhere but for above reasons, this seems to be impossible
            (If I am wrong\missing something, I'd very much like to hear about this)

        */
        let mut sdl_context = sdl2::init().video().unwrap();
       

        // hardcoded resolution for now. TODO: Implement arbitrary resolution & scaling
        let window = sdl_context.window("RustNes", 256*2, 240*2)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
        let renderer = window.renderer().build().unwrap();

        let renderer = SDLRenderer::new(renderer);

        let rom = Box::new(read_rom(rom_path));
        
        println!("{:?}", rom.header);
        
        let tv_system = rom.header.tv_system.clone();
        let mirroring = rom.header.mirroring.clone();

        let rom_mem = Rc::new(RefCell::new(rom as Box<Memory>));
        let ppu = Rc::new(RefCell::new(Ppu::new(Box::new(renderer), tv_system.clone(), mirroring, rom_mem.clone())));

        let mem = Rc::new(RefCell::new(Box::new(MemoryBus::new(rom_mem.clone(), ppu.clone())) as Box<Memory>));
        let mut console = Console {
            memory: mem.clone(),
            cpu: Cpu::new(&tv_system, mem.clone()),
            ppu: ppu.clone(),
        };


        let cpu_cycle_time_in_nanoseconds = (1.0/(console.cpu.frequency.cpu_clock_frequency/1000.0)) as u64;
        println!("CPU frequency: {}", console.cpu.frequency.cpu_clock_frequency);
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
        console.cpu.reset();

        let mut time = time::precise_time_ns();
        'main_loop: loop {
            let current_time = time::precise_time_ns();
            let time_taken = current_time - time;


            if time_taken > cpu_cycle_time_in_nanoseconds * cpu_cycles_per_tick {
                for _ in 0..cpu_cycles_per_tick {
                    console.run_emulation_tick();
                }
                time = current_time;
            }

            // TEMPORARY. Poll events so t hat window doesn't freeze

            for event in sdl_context.event_pump().poll_iter() {

                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'main_loop;
                    },
                    _ => {}
                }
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
