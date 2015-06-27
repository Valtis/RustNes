use std::fs::File;
use std::slice::Iter;
use std::iter::Peekable;
use std::io::Write;


// .byte directive currently messes this; data gets incorrectly disassembled as code
pub fn disassemble(opcodes: &Vec<u8>, file_path: &str) {
    let mut file = File::create(file_path).unwrap_or_else(|e| {
            panic!("Could not open file {}: {}", file_path, e);
            });

    let mut iter = opcodes.iter().peekable();
    loop {
        if iter.peek().is_none() {
            break;
        }
        write(&mut file, &mut iter);
    }

    file.sync_all().unwrap();
}


fn write(file: &mut File, iter :&mut Peekable<Iter<u8>>) {
    match iter.next() {
        Some(opcode) => {
            match *opcode  {
                0 => { write!(file, "brk").unwrap(); }
                1 => { write_indirect_x(file, iter, "ora"); }
                5 => { write_zero_page(file, iter, "ora"); }
                6 => { write_zero_page(file, iter, "asl"); }
                8 => { write!(file, "php").unwrap(); }
                9 => { write_immediate(file, iter, "ora"); }
                10 => { write!(file, "asl A").unwrap(); }
                13 => { write_absolute(file, iter, "ora"); }
                14 => { write_absolute(file, iter, "asl"); }
                16 => { write_relative(file, iter, "bpl"); }
                17 => { write_indirect_y(file, iter, "ora"); }
                21 => { write_zero_page_x(file, iter, "ora"); }
                22 => { write_zero_page_x(file, iter, "asl"); }
                24 => { write!(file, "clc").unwrap(); }
                25 => { write_absolute_y(file, iter, "ora"); }
                29 => { write_absolute_x(file, iter, "ora"); }
                30 => { write_absolute_x(file, iter, "asl"); }
                32 => { write_absolute(file, iter, "jsr"); }
                33 => { write_indirect_x(file, iter, "and"); }
                36 => { write_zero_page(file, iter, "bit"); }
                37 => { write_zero_page(file, iter, "and"); }
                38 => { write_zero_page(file, iter, "rol"); }
                40 => { write!(file, "plp").unwrap(); }
                41 => { write_immediate(file, iter, "and"); }
                42 => { write!(file, "rol A").unwrap(); }
                44 => { write_absolute(file, iter, "bit"); }
                45 => { write_absolute(file, iter, "and"); }
                46 => { write_absolute(file, iter, "rol"); }
                48 => { write_relative(file, iter, "bne"); }
                49 => { write_indirect_y(file, iter, "and"); }
                53 => { write_zero_page_x(file, iter, "and"); }
                54 => { write_zero_page_x(file, iter, "rol"); }
                56 => { write!(file, "sec").unwrap(); }
                57 => { write_absolute_y(file, iter, "and"); }
                61 => { write_absolute_x(file, iter, "and"); }
                62 => { write_absolute_x(file, iter, "rol"); }
                64 => { write!(file, "rti").unwrap(); }
                65 => { write_indirect_x(file, iter, "eor"); }
                69 => { write_zero_page(file, iter, "eor"); }
                70 => { write_zero_page(file, iter, "lsr"); }
                72 => { write!(file, "pha").unwrap(); }
                73 => { write_immediate(file, iter, "eor"); }
                74 => { write!(file, "lsr A", ).unwrap(); }
                76 => { write_absolute(file, iter, "jmp"); }
                77 => { write_absolute(file, iter, "eor"); }
                78 => { write_absolute(file, iter, "lsr"); }
                80 => { write_relative(file, iter, "bvc"); }
                81 => { write_indirect_y(file, iter, "eor"); }
                85 => { write_zero_page_x(file, iter, "eor"); }
                86 => { write_zero_page_x(file, iter, "lsr"); }
                88 => { write!(file, "cli").unwrap(); }
                89 => { write_absolute_y(file, iter, "eor"); }
                93 => { write_absolute_x(file, iter, "eor"); }
                94 => { write_absolute_x(file, iter, "lsr"); }
                96 => { write!(file, "rts").unwrap(); }
                97 => { write_indirect_x(file, iter, "adc"); }
                101 => { write_zero_page(file, iter, "adc"); }
                102 => { write_zero_page(file, iter, "ror"); }
                104 => { write!(file, "pla").unwrap(); }
                105 => { write_immediate(file, iter, "adc"); }
                106 => { write!(file, "ror A").unwrap(); }
                108 => { write_indirect(file, iter, "jmp"); }
                109 => { write_absolute(file, iter, "adc"); }
                110 => { write_absolute(file, iter, "ror"); }
                112 => { write_relative(file, iter, "bvs"); }
                113 => { write_indirect_y(file, iter, "adc"); }
                117 => { write_zero_page_x(file, iter, "adc"); }
                118 => { write_zero_page_x(file, iter, "ror"); }
                120 => { write!(file, "sei").unwrap(); }
                121 => { write_absolute_y(file, iter, "adc"); }
                125 => { write_absolute_x(file, iter, "adc"); }
                126 => { write_absolute_x(file, iter, "ror"); }
                129 => { write_indirect_x(file, iter, "sta"); }
                132 => { write_zero_page(file, iter, "sty"); }
                133 => { write_zero_page(file, iter, "sta"); }
                134 => { write_zero_page(file, iter, "stx"); }
                136 => { write!(file, "dey").unwrap(); }
                138 => { write!(file, "txa").unwrap(); }
                140 => { write_absolute(file, iter, "sty"); }
                141 => { write_absolute(file, iter, "sta"); }
                142 => { write_absolute(file, iter, "stx"); }
                144 => { write_relative(file, iter, "bcc"); }
                145 => { write_indirect_y(file, iter, "sta"); }
                148 => { write_zero_page_x(file, iter, "sty"); }
                149 => { write_zero_page_x(file, iter, "sta"); }
                150 => { write_zero_page_y(file, iter, "stx"); }
                152 => { write!(file, "tya").unwrap(); }
                153 => { write_absolute_y(file, iter, "sta"); }
                154 => { write!(file, "txs").unwrap(); }
                157 => { write_absolute_x(file, iter, "sta"); }
                160 => { write_immediate(file, iter, "ldy"); }
                161 => { write_indirect_x(file, iter, "lda"); }
                162 => { write_immediate(file, iter, "ldx"); }
                164 => { write_zero_page(file, iter, "ldy"); }
                165 => { write_zero_page(file, iter, "lda"); }
                166 => { write_zero_page(file, iter, "ldx"); }
                168 => { write!(file, "tay").unwrap();}
                169 => { write_immediate(file, iter, "lda"); }
                170 => { write!(file, "tax").unwrap();}
                172 => { write_absolute(file, iter, "ldy"); }
                173 => { write_absolute(file, iter, "lda"); }
                174 => { write_absolute(file, iter, "ldx"); }
                176 => { write_relative(file, iter, "bcs"); }
                177 => { write_indirect_y(file, iter, "lda"); }
                180 => { write_zero_page_x(file, iter, "ldy"); }
                181 => { write_zero_page_x(file, iter, "lda"); }
                182 => { write_zero_page_y(file, iter, "ldx"); }
                184 => { write!(file, "clv").unwrap(); }
                185 => { write_absolute_y(file, iter, "lda"); }
                186 => { write!(file, "tsx").unwrap(); }
                188 => { write_absolute_x(file, iter, "ldy"); }
                189 => { write_absolute_x(file, iter, "lda"); }
                190 => { write_absolute_y(file, iter, "ldx"); }
                192 => { write_immediate(file, iter, "cpy"); }
                193 => { write_indirect_x(file, iter, "cmp"); }
                196 => { write_zero_page(file, iter, "cpy"); }
                197 => { write_zero_page(file, iter, "cmp"); }
                198 => { write_zero_page(file, iter, "dec"); }
                200 => { write!(file, "iny").unwrap(); }
                201 => { write_immediate(file, iter, "cmp"); }
                202 => { write!(file, "dex").unwrap(); }
                204 => { write_absolute(file, iter, "cpy"); }
                205 => { write_absolute(file, iter, "cmp"); }
                206 => { write_absolute(file, iter, "dec"); }
                208 => { write_relative(file, iter, "bne"); }
                209 => { write_indirect_y(file, iter, "cmp"); }
                213 => { write_zero_page_x(file, iter, "cmp"); }
                214 => { write_zero_page_x(file, iter, "dec"); }
                216 => { write!(file, "cld").unwrap(); }
                217 => { write_absolute_y(file, iter, "cmp"); }
                221 => { write_absolute_x(file, iter, "cmp"); }
                222 => { write_absolute_x(file, iter, "dex"); }
                224 => { write_immediate(file, iter, "cpx"); }
                225 => { write_indirect_x(file, iter, "sbc"); }
                228 => { write_zero_page(file, iter, "cpx"); }
                229 => { write_zero_page(file, iter, "sbc"); }
                230 => { write_zero_page(file, iter, "inc"); }
                232 => { write!(file, "inx").unwrap(); }
                233 => { write_immediate(file, iter, "sbc"); }
                234 => { write!(file, "nop").unwrap(); }
                236 => { write_absolute(file, iter, "cpx"); }
                237 => { write_absolute(file, iter, "sbc"); }
                238 => { write_absolute(file, iter, "inc"); }
                240 => { write_relative(file, iter, "beq"); }
                241 => { write_indirect_y(file, iter, "sbc"); }
                245 => { write_zero_page_x(file, iter, "sbc");}
                246 => { write_zero_page_x(file, iter, "inc"); }
                248 => { write!(file, "sed").unwrap(); }
                249 => { write_absolute_y(file, iter, "sbc"); }
                253 => { write_absolute_x(file, iter, "sbc"); }
                254 => { write_absolute_x(file, iter, "inc"); }
                _ =>  { write!(file, "Unrecognized opcode: {}", opcode).unwrap(); }
            }
            write!(file, "\n").unwrap();
        },
        None => {}
    }
}


fn write_absolute(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    let low_byte = *iter.next().unwrap();
    let high_byte = *iter.next().unwrap();

    let operand:u16 = ((high_byte as u16) << 8) | low_byte as u16;
    write!(file, "{} ${:x}", instruction, operand).unwrap();
}

fn write_absolute_x(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    write_absolute(file, iter, instruction);
    write!(file, ",x").unwrap();
}

fn write_absolute_y(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    write_absolute(file, iter, instruction);
    write!(file, ",y").unwrap();
}

fn write_indirect(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    let low_byte = *iter.next().unwrap();
    let high_byte = *iter.next().unwrap();

    let operand:u16 = ((high_byte as u16) << 8) | low_byte as u16;
    write!(file, "{} (${:x})", instruction, operand).unwrap();
}

fn write_indirect_x(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    let low_byte = *iter.next().unwrap();
    let high_byte = *iter.next().unwrap();

    let operand:u16 = ((high_byte as u16) << 8) | low_byte as u16;
    write!(file, "{} (${:x},X)", instruction, operand).unwrap();
}

fn write_indirect_y(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    let low_byte = *iter.next().unwrap();
    let high_byte = *iter.next().unwrap();

    let operand:u16 = ((high_byte as u16) << 8) | low_byte as u16;
    write!(file, "{} (${:x},Y)", instruction, operand).unwrap();
}

fn write_immediate(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    write!(file, "{} #${:x}", instruction, *iter.next().unwrap()).unwrap();
}

fn write_relative(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    write!(file, "{} ${:x}", instruction, *iter.next().unwrap()).unwrap();
}

fn write_zero_page(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    write!(file, "{} ${:x}", instruction, *iter.next().unwrap()).unwrap();
}

fn write_zero_page_x(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    write!(file, "{} ${:x},X", instruction, *iter.next().unwrap()).unwrap();
}
fn write_zero_page_y(file: &mut File, iter :&mut Peekable<Iter<u8>>, instruction: &str) {
    write!(file, "{} ${:x},Y", instruction, *iter.next().unwrap()).unwrap();
}
