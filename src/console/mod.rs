extern crate time;
extern crate sdl2;
use self::sdl2::Sdl;
use self::sdl2::render::{Canvas, TextureCreator};
use self::sdl2::video::{Window, WindowContext};
use self::sdl2::keyboard::Keycode;
use self::sdl2::event::Event;

use memory::Memory;
use memory_bus::*;
use cpu::Cpu;
use ppu::Ppu;
use rom::read_rom;
use ppu::renderer::*;
use controller::Controller;

use std::rc::Rc;
use std::cell::RefCell;

struct Console<'a> {
    cpu: Cpu<'a>,
    ppu: Rc<RefCell<Ppu<'a>>>,
    controllers: Vec<Rc<RefCell<Controller>>>,
}
// borrow checker workarounds
struct CanvasStruct {
    canvas: Canvas<Window>,
}



fn initialize_sdl() -> (Sdl, CanvasStruct, TextureCreator<WindowContext>) {
    let sdl_context = sdl2::init()
        .unwrap_or_else(|e| panic!("Failed to initialize SDL context"));

    let video_subsystem = sdl_context
        .video().unwrap_or_else(|e| panic!("Failed to initialize SDL video subsystem"));

    // hardcoded resolution for now. TODO: Implement arbitrary resolution & scaling
    let window = video_subsystem.window("RustNes", 256*2, 240*2)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    // TODO: Solve lifetime issues with initialization
    let canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    (sdl_context, CanvasStruct { canvas: canvas }, texture_creator)
}

fn initialize_console<'a>(
    rom_path: &str,
    canvas: &'a mut CanvasStruct,
    texture_creator: &'a TextureCreator<WindowContext>) -> Console<'a> {
    let rom = Box::new(read_rom(rom_path));

    let controller_one = Rc::new(RefCell::new(Controller::new(None)));
    let controller_two = Rc::new(RefCell::new(Controller::new(None)));
    let controllers = vec![controller_one.clone(), controller_two.clone()];


    println!("{:#?}", rom.header);

    let tv_system = rom.header.tv_system.clone();
    let mirroring = rom.header.mirroring.clone();

    let rom_mem = Rc::new(RefCell::new(rom as Box<Memory>));
    let ppu = Rc::new(RefCell::new(
        Ppu::new(
            Renderer::new(
                &mut canvas.canvas,
                &texture_creator),
            tv_system.clone(),
            mirroring,
            rom_mem.clone())));

    let mem = Rc::new(RefCell::new(
        Box::new(
            MemoryBus::new(
                rom_mem.clone(),
                ppu.clone(),
                controllers.clone(),
            )
        ) as Box<Memory>));

    Console {
        cpu: Cpu::new(&tv_system, mem.clone()),
        ppu: ppu.clone(),
        controllers: controllers.clone(),
    }
}

pub fn execute(rom_path: &str) {
    let (sdl_context, mut canvas, texture_creator) = initialize_sdl();
    let mut console = initialize_console(rom_path, &mut canvas, &texture_creator);

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

        let mut event_pump = sdl_context.event_pump().unwrap();
        for event in event_pump.poll_iter() {

            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                Event::KeyDown { keycode, ..} => {
                    if let Some(key) = keycode {
                        console.controllers[0].borrow_mut().key_down(key);
                        console.controllers[1].borrow_mut().key_down(key);
                    }
                },
                Event::KeyUp { keycode, ..} => {
                    if let Some(key) = keycode {
                        console.controllers[0].borrow_mut().key_up(key);
                        console.controllers[1].borrow_mut().key_up(key);
                    }
                }
                _ => {}
            }
        }
    }
}

impl<'a> Console<'a> {
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
