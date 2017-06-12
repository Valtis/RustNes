
mod console;
mod apu;
mod cpu;
mod ppu;
mod memory;
mod memory_bus;
mod ram;
mod rom;
mod controller;

use std::env;

fn main() {
    let args : Vec<_> = env::args().collect();
    if args.len() == 1 {
        println!("Program name expected as cmd line arg");
        return;
    }
    console::execute(&args[1]);
}
