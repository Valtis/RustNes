extern crate nes;

use nes::rom_reader::read_rom;
use nes::cpu::get_cpu;

fn main() {
    let rom = read_rom("supermario.nes");
    println!("{:?}", get_cpu(rom));
}
