mod console;
mod cpu;
mod ppu;
mod memory;
mod memory_bus;
mod ram;
mod rom;

fn main() {
    let mut console = console::Console::new("nestest.nes");
    console.execute();
}
