extern crate nes;

use nes::rom_reader::read_rom;
use nes::cpu::get_cpu;
use nes::disassembler::disassemble;
fn main() {
    let rom = read_rom("supermario.nes");
    disassemble(&rom.prg_rom_data, "disassembly.asm");

    // println!("{:?}", get_cpu(rom));
}
