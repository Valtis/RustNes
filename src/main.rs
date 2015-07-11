
mod console;
mod cpu;
mod ppu;
mod memory;
mod memory_bus;
mod ram;
mod rom;

use console::Console;

fn main() {
    Console::execute("donkey.nes");
}
