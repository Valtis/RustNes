mod console;
mod cpu;
mod memory;
mod rom;

fn main() {
    let mut console = console::Console::new("nestest.nes");
    console.execute();
}
