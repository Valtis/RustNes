extern crate nes;

use nes::console::Console;

fn main() {
    let mut console = Console::new("nestest.nes");
    console.execute();
}
