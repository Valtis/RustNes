mod vram;
mod tv_system_values;
pub mod renderer;


use memory::Memory;
use rom::*;
use self::vram::Vram;
use self::tv_system_values::TvSystemValues;
use self::renderer::Renderer;
use self::renderer::Pixel;

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

static PALETTE: [u8; 192] = [
    124,124,124,    0,0,252,        0,0,188,        68,40,188,
    148,0,132,      168,0,32,       168,16,0,       136,20,0,
    80,48,0,        0,120,0,        0,104,0,        0,88,0,
    0,64,88,        0,0,0,          0,0,0,          0,0,0,
    188,188,188,    0,120,248,      0,88,248,       104,68,252,
    216,0,204,      228,0,88,       248,56,0,       228,92,16,
    172,124,0,      0,184,0,        0,168,0,        0,168,68,
    0,136,136,      0,0,0,          0,0,0,          0,0,0,
    248,248,248,    60,188,252,     104,136,252,    152,120,248,
    248,120,248,    248,88,152,     248,120,88,     252,160,68,
    248,184,0,      184,248,24,     88,216,84,      88,248,152,
    0,232,216,      120,120,120,    0,0,0,          0,0,0,
    252,252,252,    164,228,252,    184,184,248,    216,184,248,
    248,184,248,    248,164,192,    240,208,176,    252,224,168,
    248,216,120,    216,248,120,    184,248,184,    184,248,216,
    0,252,252,      248,216,248,    0,0,0,          0,0,0
];


struct SpriteRenderData {
    is_sprite_0: bool,
    has_foreground_priority: bool,
    palette_index: u8,
}

impl SpriteRenderData {
    fn new(is_sprite_0: bool, has_foreground_priority: bool, palette_index: u8) -> SpriteRenderData {
        SpriteRenderData {
            is_sprite_0: is_sprite_0,
            has_foreground_priority: has_foreground_priority,
            palette_index: palette_index,
        }
    }
}

pub struct Ppu {
    object_attribute_memory: Vec<u8>,
    secondary_oam: Vec<u8>,
    secondary_contains_sprite_0: bool,
    vram: Box<Memory>,
    registers: Registers,
    address_latch: bool,
    vram_address: u16,
    fine_x_scroll: u8,
    vram_read_buffer: u8,
    tv_system: TvSystemValues,
    current_scanline: u16,
    pos_at_scanline: u16,
    nmi_occured: bool,
    name_table_byte: u8,
    attribute_table_byte: u8,
    pattern_table_low_byte: u8,
    pattern_table_high_byte: u8,
    background_data: u64,
    pixels: Vec<Pixel>,
    renderer: Box<Renderer>,
}



impl fmt::Debug for Ppu{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:?}", self.registers));
        write!(f, "<Memory contents not included>")
    }
}

impl Memory for Ppu {
    fn read(&mut self, cpu_address: u16) -> u8 {
        match cpu_address & 0x0007 {
            0 => panic!("Attempting to read from write-only ppu control register (address 0x{:04X})", cpu_address),
            1 => panic!("Attempting to read from write-only ppu mask register (address 0x{:04X})", cpu_address),
            2 => self.status_register_read(),
            3 => panic!("Attempting to read from write-only ppu oam address register (address 0x{:04X})", cpu_address),
            4 => self.oam_data_register_read(),
            5 => panic!("Attempting to read from write-only ppu scroll register (address 0x{:04X})", cpu_address),
            6 => panic!("Attempting to read from write-only ppu address register (address 0x{:04X})", cpu_address),
            7 => self.ppu_data_register_read(),
            _ => panic!("Something went horribly wrong in modulus calculation in ppu.read (address: {})", cpu_address),
        }
    }

    fn write(&mut self, cpu_address: u16, value: u8) {
        match cpu_address & 0x0007 {
            0 => self.control_register_write(value),
            1 => self.registers.mask = value,
            2 => panic!("Attempting to write to read-only ppu status register (address 0x{:04X})", cpu_address),
            3 => self.registers.oam_address = value,
            4 => self.oam_data_register_write(value),
            5 => self.scroll_register_write(value),
            6 => self.ppu_address_register_write(value),
            7 => self.ppu_data_register_write(value),
            _ => panic!("Something went horribly wrong in modulus calculation in ppu.write (address: {})", cpu_address),
        }
    }
}

impl Ppu {
    pub fn new(renderer: Box<Renderer>, tv_system: TvSystem, mirroring: Mirroring, rom: Rc<RefCell<Box<Memory>>>) -> Ppu {
        Ppu {
            object_attribute_memory: vec![0;256],
            secondary_oam: vec![0;32],
            secondary_contains_sprite_0: false,
            vram: Box::new(Vram::new(mirroring, rom)),
            registers: Registers::new(),
            address_latch: false,
            vram_address: 0,
            fine_x_scroll: 0,
            vram_read_buffer: 0,
            tv_system: TvSystemValues::new(&tv_system),
            current_scanline: 0,
            pos_at_scanline: 0,
            nmi_occured: false,
            name_table_byte: 0,
            attribute_table_byte: 0,
            pattern_table_low_byte: 0,
            pattern_table_high_byte: 0,
            background_data: 0,
            pixels: vec![Pixel::new(0,0,0);240*256],
            renderer: renderer,
        }
    }

    fn generate_nmi_if_flags_set(&mut self) {
        if self.registers.control & 0x80 == 0x80 && self.registers.status & 0x80 == 0x80 {
            self.nmi_occured = true;
        }
    }

    pub fn nmi_occured(&mut self) -> bool {
        let occured = self.nmi_occured;
        self.nmi_occured = false;
        occured
    }

    fn increment_vram(&mut self) {
        if self.registers.control & 0x04 == 0 {
            self.vram_address += 1;
        }  else {
            self.vram_address += 32;
        }
    }

    fn control_register_write(&mut self, value: u8) {
        self.registers.control = value;
        self.generate_nmi_if_flags_set();
    }

    fn status_register_read(&mut self) -> u8 {
        self.address_latch = false;
        let val = self.registers.status;
        self.registers.status = self.registers.status & 0x7F;
        val
    }

    fn oam_data_register_read(&mut self) -> u8 {
        let address = self.registers.oam_address as usize;
        self.object_attribute_memory[address]
    }

    fn oam_data_register_write(&mut self, value: u8) {
        let address = self.registers.oam_address as usize;
        self.registers.oam_address += 1;
        self.object_attribute_memory[address] = value;
    }

    fn scroll_register_write(&mut self, value: u8) {
        if self.address_latch == false {
        /*
            $2005 first write (w is 0)

            t: ....... ...HGFED = d: HGFED...
            x:              CBA = d: .....CBA
            w:                  = 1

            Where w is the address latch, d is data to be written, x is the fine scroll value and
            t is the 15 bit temporary register
        */
            self.fine_x_scroll = value & 0x07;
            self.registers.temporary = (self.registers.temporary & 0xFFE0) | (value as u16) >> 3; // clear bits
        } else {
        /*
            $2005 second write (w is 1)

            t: CBA..HG FED..... = d: HGFEDCBA
            w:                  = 0
        */
            self.registers.temporary = (self.registers.temporary & 0x0FFF) | ((value as u16) & 0x0007) << 12;
            self.registers.temporary = (self.registers.temporary & 0xFC1F) | ((value as u16) & 0x00F8) << 2;
        }
        self.address_latch = !self.address_latch;
    }

    fn ppu_address_register_write(&mut self, value: u8) {
        if self.address_latch == false {
            /*
                $2006 first write (w is 0)

                t: .FEDCBA ........ = d: ..FEDCBA
                t: X...... ........ = 0
                w:                  = 1

                Where w is the address latch, d is data to be written and t is the 15 bit temporary
                register
            */

            self.registers.temporary = self.registers.temporary & 0x80FF; // clear bits
            self.registers.temporary = self.registers.temporary | ((value & 0x3F) as u16) << 8;

        } else {
            /*
                $2006 second write (w is 1)

                t: ....... HGFEDCBA = d: HGFEDCBA
                v                   = t
                w:                  = 0
                As above. v is the vram address register
            */
            self.registers.temporary = self.registers.temporary & 0xFF00; // clear low 8 bytes
            self.registers.temporary = self.registers.temporary | value as u16;
            self.vram_address = self.registers.temporary;
        }
        self.address_latch = !self.address_latch;
    }

    fn ppu_data_register_read(&mut self) -> u8 {
        let address = self.vram_address & 0x7FFF;
        self.increment_vram();

        if address <= 0x3EFF {
            // read is buffered; return value in buffer and update buffer to value at address
            let buffer = self.vram_read_buffer;
            self.vram_read_buffer = self.vram.read(address);
            buffer
        } else {
            // read is not buffered; update buffer to value at address - 0x1000
            let value = self.vram.read(address);
            self.vram_read_buffer = self.vram.read(address - 0x1000);
            value
        }
    }

    fn ppu_data_register_write(&mut self, value: u8) {
        let address = self.vram_address;

        self.increment_vram();
        self.vram.write(address, value);
    }

    pub fn oam_dma_write(&mut self, data: Vec<u8>) {
        if data.len() != 256 {
            panic!("Invalid oam data write length: 256 expected but was {}", data.len());
        }
        self.object_attribute_memory = data;
    }

    // how many cycles will be executed this cpu cycle
    // also updates counters
    fn get_cycle_count(&mut self) -> u8 {
        if self.tv_system.ppu_extra_cycle_every_cpu_cycle != 0 {
            self.tv_system.extra_cycle_counter += 1;
            if self.tv_system.extra_cycle_counter == self.tv_system.ppu_extra_cycle_every_cpu_cycle {
                self.tv_system.extra_cycle_counter = 0;
                self.tv_system.ppu_cycles_per_cpu_cycle + 1
            } else {
                self.tv_system.ppu_cycles_per_cpu_cycle
            }
        } else {
            self.tv_system.ppu_cycles_per_cpu_cycle
        }
    }

    pub fn execute_cycles(&mut self) {
        let cycles = self.get_cycle_count();
        for _ in 0..cycles {
            self.execute_cycle();
        }
    }

    fn update_scanline_pos(&mut self) {
        self.pos_at_scanline += 1;
        if self.pos_at_scanline == 340 {
            self.pos_at_scanline = 0;
            self.current_scanline += 1;
            if self.current_scanline > 261 {
                self.current_scanline = 0;
            }
        }
    }

    fn execute_cycle(&mut self) {

        let rendered_scanlines = 240;

        let render_end = self.tv_system.vblank_frames + rendered_scanlines;
        let post_render_end = render_end + self.tv_system.post_render_scanlines;

        if self.current_scanline < self.tv_system.vblank_frames {
            self.do_vblank();
        } else if self.current_scanline == self.tv_system.vblank_frames {
            self.do_pre_render_line();
        } else if self.current_scanline <= render_end {
            self.do_render_line();
        } else if self.current_scanline <= post_render_end {
            // post render line - do nothing ppu wise. As rendering has ended, we can actually render the image
            if self.pos_at_scanline == 0 {
                self.renderer.render(&self.pixels); // placeholder
            }
        }

        self.update_scanline_pos();
    }


    fn rendering_enabled(&mut self) -> bool {
        (self.registers.mask & 0x18) != 0
    }


    fn do_vblank(&mut self) {
        // VBLANK - do not access memory or render.
        // However, set vblank flag on second tick of first scanline and raise NMI if nmi flag is set
        // also clear the overflow flag and sprite 0 hit
        if self.current_scanline == 0 && self.pos_at_scanline == 1 {
            self.registers.status = (self.registers.status & 0x9F) | 0x80;
            self.generate_nmi_if_flags_set();
        }
    }

    fn do_pre_render_line(&mut self) {
        if self.pos_at_scanline == 1 { // unset vblank flag on second tick
            self.registers.status = self.registers.status & 0x7F;
        }

        if self.rendering_enabled() {
            if (self.pos_at_scanline >=1 && self.pos_at_scanline <= 256) || (self.pos_at_scanline >= 321 && self.pos_at_scanline <= 336) {
                self.do_memory_access();
            }

            if self.pos_at_scanline == 256 {
                self.increment_vram_y();
            } else if self.pos_at_scanline == 257  {
                self.update_x_scroll();
            }

            // reset scroll values
            if self.pos_at_scanline >= 280 && self.pos_at_scanline <= 304 && self.rendering_enabled() {
                self.update_y_scroll();
            }
            // in real NES, sprite evaluation happens at the same time than background evaluation
            // however, whereas cycle accuracy with background is required for split screen emulation
            // I am unaware as of writing this of any downsides of doing sprite evaluation in single pass.
            // This may backfire horribly later on
            if self.pos_at_scanline == 256 {
                self.evaluate_sprites();
            }
        }
    }



    fn do_render_line(&mut self) {
        if !self.rendering_enabled() {
            // do nothing if rendering is not enabled
            return;
        }

        if self.pos_at_scanline == 0 {
            // idle cycle
        } else if self.pos_at_scanline <= 256 {
            self.render_pixel();
            self.do_memory_access();
            if self.pos_at_scanline == 256 {
                self.increment_vram_y();
            }
        } else if self.pos_at_scanline <= 320 {
            // background wise do nothing - sprite part is handled in evaluate_sprites
            // Actual nes recycles circuitry and sprites use same tile
        } else if self.pos_at_scanline <= 336 {
            self.do_memory_access();
        }

        // as with pre-render line
        if self.pos_at_scanline == 256 {
            self.evaluate_sprites();
        }

        if self.pos_at_scanline == 257  {
            self.update_x_scroll();
        }

    }

    fn do_memory_access(&mut self) {
         self.background_data = self.background_data << 4; // shift data for next pixel rendering
         match self.pos_at_scanline & 0x07 {
            0 => {
                if self.pos_at_scanline & 0x07 == 0 {
                    self.increment_vram_x();
                }
                self.update_background_data();
            },
            1 => self.read_nametable_byte(),
            3 => self.read_attribute_byte(),
            5 => self.read_pattern_table_low_byte(),
            7 => self.read_pattern_table_high_byte(),
            _ => {} // do nothing
        }
    }

    // copy data that have been read to render buffer
    fn update_background_data(&mut self) {
    	let mut temp: u32 = 0;
    	for i in 0..8 {
    		// get color bits from low\high bytes of pattern table data (these are split into separate bytes for
            // reasons I do not understand; must have been hardware constraint back in the 80's)

            // bit 0
    		let mut color = ((self.pattern_table_low_byte << i) & 0x80) >> 7;
            // and bit 1
    		color = color  | ((self.pattern_table_high_byte << i) & 0x80) >> 6;

            // attribute table contains the palette bits, it's either 0x04, 0x08 or 0x0C
            // due to the bitshift left done when fetching the value
    		temp = (temp << 4) | self.attribute_table_byte as u32 | color as u32;
    	}
    	self.background_data = self.background_data | temp as u64;
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching for address calculations
    fn read_nametable_byte(&mut self) {
        let address = 0x2000 | (self.vram_address & 0x0FFF);
        self.name_table_byte = self.vram.read(address);
    }
    /*
        address can be interpreted as follows

            yyy NN YYYYY XXXXX
            ||| || ||||| +++++-- coarse X scroll
            ||| || +++++-------- coarse Y scroll
            ||| ++-------------- nametable select
            +++----------------- fine Y scroll
    */

    fn read_attribute_byte(&mut self) {
        // each attribute table is stored in the final 64 bytes of a nametable
        // 0x23C0 is the location of the first attribute table.
        // correct attribute table is then selected by doing some bit manipulation to the vram address
        let attribute_address = 0x23C0
                | (self.vram_address & 0x0C00)
                | ((self.vram_address >> 4) & 0x38)
                | ((self.vram_address >> 2) & 0x07);

        // each byte contains 4 palettes (2 bits per palette; thus the maximum of 4 palettes)
        // the byte covers 32x32 pixel square, which itself consits of 4 16x16 pixel squares. A palette
        // is assigned to this 16x16 square, which is also the reason why 16x16 pixel squares must
        // share the palette.
        // (see http://wiki.nesdev.com/w/index.php/PPU_attribute_tables)

        // Thus we need to do some bit manipulation in order to select the correct palette.
        // Then the attribute is finally shifted 2 position left so that the update_background_data
        // can just straight bitwise_or it in the final data buffer (otherwise the << 2 would
        // have to be done there)
        let shift = ((self.vram_address >> 4) & 0x04) | (self.vram_address & 0x02);
        self.attribute_table_byte = ((self.vram.read(attribute_address) >> shift) & 0x03) << 2;
    }

    fn calculate_pattern_table_address(&mut self) -> u16 {
        // get fine y bits from vram address (bits 12, 13 & 14)
        let fine_y = (self.vram_address >> 12) & 0x07;
        // select correct pattern table (if bit 4 at control is 0 -> table at 0x000, 1 -> table at 0x1000)
        let table = 0x1000 * (((self.registers.control as u16) & 0x10) >> 4);
        // nametable byte selects tile in pattern table; tile is 16 bytes
        table + (self.name_table_byte as u16)*16 + fine_y
    }

    fn read_pattern_table_low_byte(&mut self) {
        let address = self.calculate_pattern_table_address();
        self.pattern_table_low_byte = self.vram.read(address);
    }

    fn read_pattern_table_high_byte(&mut self) {
        // offset for high byte
        let address = self.calculate_pattern_table_address() + 8;
        self.pattern_table_high_byte = self.vram.read(address);
    }



    // TODO - implement color emphasis
    fn render_pixel(&mut self) {
        // for now, only background rendering.

        let y = self.current_scanline - self.tv_system.vblank_frames - 1; // - 1 for pre-render-line
        let x = self.pos_at_scanline - 1; // - 1 for the skipped cycle

        let background = self.get_background_for_rendering(x);
        let sprite = self.get_sprite_for_rendering(x, y);

        let sprite_multiplex = sprite.palette_index % 4;
        let background_multiplex = background % 4;
        // http://wiki.nesdev.com/w/index.php/PPU_rendering#Preface - see priority multiplexer decision table
        let palette_index = if background_multiplex == 0 && sprite_multiplex == 0 {
            0
        } else if sprite_multiplex == 0 && background_multiplex != 0 {
            background
        } else if sprite_multiplex != 0 && background_multiplex == 0 {
            sprite.palette_index
        } else {
            if sprite.is_sprite_0 {
                self.registers.status = self.registers.status | 0x40;
            }

            if sprite.has_foreground_priority {
                sprite.palette_index
            } else {
                background
            }
        };

        let color_index = (self.vram.read(0x3F00 + palette_index as u16) % 64) as usize;

        let index = y as usize*256 + x as usize;
        self.pixels[index] = Pixel::new(PALETTE[color_index*3], PALETTE[color_index*3 + 1], PALETTE[color_index*3 + 2]);
    }

    fn get_background_for_rendering(&mut self, x: u16) -> u8{
        if self.registers.mask & 0x02 == 0 && x < 8 { // background rendering disabled for first 8 pixels
           0
       } else {
           // if background rendering is disabled, return 0
           if self.registers.mask & 0x08 == 0 {
               0
           } else {
               // fetch the current data from buffer by first shifting 32 bits left (discard first 4 bytes)
               // then further shifting it by 0 - 7 pixels depending on fine scroll value and finally getting
               // the first 4 bits
               ((((self.background_data >> 32) as u32) >> ((7 - self.fine_x_scroll)*4)) & 0x0F) as u8
           }
       }
    }

    fn get_sprite_for_rendering(&mut self, x: u16, y: u16) -> SpriteRenderData {
        if self.registers.mask & 0x04 == 0 && x < 8 { // sprite rendering disabled for first 8 pixels
            SpriteRenderData::new(false, false, 0)
        } else {
            if self.registers.mask & 0x10 == 0 {
                SpriteRenderData::new(false, false, 0) // sprite rendering is disabled
            } else {
                // 16 8x16 sprites not implemented yet
                if self.registers.control & 0x20 != 0 {
                    panic!("8x16 sprites are not implemented yet");
                }
                // first non-transparent pixel is selected for rendering
                for i in 0..8 {
                    let sprite_y = self.secondary_oam[i*4 + 0];
                    let sprite_patten_index = self.secondary_oam[i*4 + 1] as u16;
                    let sprite_attribute = self.secondary_oam[i*4 + 2];
                    let sprite_begin_x = self.secondary_oam[i*4 + 3] as u16;

                    // in range && not transparent
                    if x >= sprite_begin_x && x < sprite_begin_x + 8 {
                        let x_diff = x - sprite_begin_x;

                        // horizonal offset
                        let x_shift = if sprite_attribute & 0x40 == 0 {
                            x_diff
                        } else {
                            7 - x_diff
                        };

                        let y_diff = y - (sprite_y) as u16;
                        let sprite_y_offset = if sprite_attribute & 0x80 == 0{
                            y_diff
                        } else {
                            7 - y_diff
                        };

                        // TODO - consider reusing the pattern table fetching code from background logic
                        // pattern table
                        //let table = 0x1000 * (((self.registers.control as u16) & 0x10) >> 4);
                        //println!("table: {}", table);
                        let table = 0;
                        let low_byte = self.vram.read(table + sprite_patten_index*16 + sprite_y_offset);
                        let high_byte = self.vram.read(table + sprite_patten_index*16 + 8 + sprite_y_offset);

                		let mut color = ((low_byte << x_shift) & 0x80) >> 7;
                		color = color  | ((high_byte << x_shift) & 0x80) >> 6;
                        if color == 0 {
                            continue;
                        }
                        let sprite_palette_offset = 4*4;
                        let palette_index = sprite_palette_offset
                            + ((sprite_attribute & 0x03) << 2 | color);

                        return SpriteRenderData::new(i == 0 && self.secondary_contains_sprite_0,
                            sprite_attribute & 0x20 == 0,
                            palette_index);
                    }
                }
                SpriteRenderData::new(false, false, 0)
            }
        }
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#Coarse_X_increment
    fn increment_vram_x(&mut self) {
    	if self.vram_address & 0x001F == 31 {
            self.vram_address = self.vram_address & 0xFFE0;          // coarse X = 0
            self.vram_address ^= 0x0400;           // switch horizontal nametable
    	} else {
    		self.vram_address += 1;
    	}
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#Y_increment
    // Implementation directly from nesdev wiki with only necessary changes to make it compile.
    // Commments preserved

    // Executed every 256 pixel on (pre)render scanlines if rendering is enabled
    fn increment_vram_y(&mut self) {
         if (self.vram_address & 0x7000) != 0x7000 {        // if fine Y < 7
            self.vram_address += 0x1000;                      // increment fine Y
        }
        else {
            self.vram_address &= !0x7000;                     // fine Y = 0
            let mut y = (self.vram_address & 0x03E0) >> 5;        // let y = coarse Y
            if y == 29 {
                y = 0;                          // coarse Y = 0
                self.vram_address ^= 0x0800;                    // switch vertical nametable
            } else if y == 31 {
                y = 0;                          // coarse Y = 0, nametable not switched
            } else {
                y += 1;
            }                         // increment coarse Y
            self.vram_address = (self.vram_address & !0x03E0) | (y << 5);     // put coarse Y back into v
        }
    }

    fn update_x_scroll(&mut self) {
       //  v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
	   self.vram_address = (self.vram_address & 0xFBE0) | (self.registers.temporary & 0x041F);
    }

    fn update_y_scroll(&mut self) {
        // v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
        self.vram_address = (self.vram_address & 0x841F) | (self.registers.temporary & 0x7BE0);
    }

    // http://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
    // For ease of implementation, I'm not going for cycle accuracy here.
    // This will be changed in case this actually causes issues.
    fn evaluate_sprites(&mut self) {
        self.initialize_secondary_oam();
        self.copy_data_to_secondary_oam();
    }

    fn initialize_secondary_oam(&mut self) {
        // initialize secondary oam
        for i in 0..32 {
            self.secondary_oam[i] = 0xFF;
        }
        self.secondary_contains_sprite_0 = false;
    }

    fn copy_data_to_secondary_oam(&mut self) {
        let mut secondary_oam_sprites = 0;
        // used to emulate the PPU overflow flag bug where offset is incorrectly incremented
        let mut overflow_offset = 0;
        for oam_sprite in 0..64 {
            if secondary_oam_sprites < 8 {
                self.evaluate_sprite_for_addition(oam_sprite, &mut secondary_oam_sprites);
            } else {
                // NES PPU has a hardware bug when handling overflow flag; it is supposed to scan
                // the remaining sprite y coordinates and set the overflow flag if additional sprites
                // are on the scanline. However the cirucitry incorrectly increments the offset
                // and thus the result is more or less random

                // for what it's worth, every 4th sprite is evaluated correctly
                let incorrect_index = (oam_sprite * 4 + overflow_offset) as usize;
                let incorrect_y_coordinate = self.object_attribute_memory[incorrect_index];

                if self.sprite_is_on_scanline(incorrect_y_coordinate) {
                    self.registers.status = self.registers.status | 0x20;
                    break;
                }
                // incorrect PPU offset increment
                overflow_offset = (overflow_offset + 1) & 0x03; // wraps around
            }
        }
    }

    fn evaluate_sprite_for_addition(&mut self, oam_sprite: u8, secondary_oam_sprites: &mut u8) {
        let index = (oam_sprite * 4) as usize;
        let y = self.object_attribute_memory[index];
        let secondary_index = (*secondary_oam_sprites*4) as usize;

        // y is written to secondary oam in any case (if there is space), even if sprite is not visible
        // in case there are fewer than 8 sprites on scanline, this will be the y value of sprite 63
        if secondary_index < self.secondary_oam.len() {
            self.secondary_oam[secondary_index] = self.object_attribute_memory[index];
        }


        if self.sprite_is_on_scanline(y) {
            if oam_sprite == 0 {
                self.secondary_contains_sprite_0 = true;
            }
            // copy remaining bytes into secondary oam
            for i in 1..4 {
                self.secondary_oam[secondary_index + i] = self.object_attribute_memory[index + i];
            }
            *secondary_oam_sprites += 1;
        }
    }

    fn sprite_is_on_scanline(&mut self, y: u8) -> bool {
        let diff = (self.current_scanline - self.tv_system.vblank_frames) as i16 - y as i16;
        // sprite height; either 8 pixels or 16 pixels, depending on whether bit 5 is set
        // in the control register
        let height = 8 + 8*((0x20 & self.registers.control) >> 5) as i16;

        // sprite is visible if diff is between 0 and height
        diff >= 0 && diff < height
    }
}

#[derive(Debug)]
struct Registers {
    control: u8,
    mask: u8,
    status: u8,
    oam_address: u8,
    oam_dma: u8,
    temporary: u16, // stores temporary values when writing to 0x2005/0x2006
}

impl Registers {
    fn new() -> Registers {
        Registers {
            control: 0,
            mask: 0,
            status: 0,
            oam_address: 0,
            oam_dma: 0,
            temporary: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;
    use rom::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use super::renderer::*;
    struct MockMemory {
        memory: Vec<u8>
    }

    impl MockMemory {
        fn new() -> MockMemory {
            MockMemory {
                memory: vec![0;0xFFFF + 1],
            }
        }
    }

    impl Memory for MockMemory {
        fn read(&mut self, address: u16) -> u8 {
            self.memory[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.memory[address as usize] = value;
        }
    }

    struct MockRenderer;

    impl MockRenderer {
        fn new() -> MockRenderer {
            MockRenderer
        }
    }

    impl Renderer for MockRenderer {
        fn render(&mut self, pixels: &Vec<Pixel>) {

        }
    }

    fn create_test_ppu() -> Ppu {
        let rom = Rc::new(RefCell::new(Box::new(MockMemory::new()) as Box<Memory>));
        let mut ppu = Ppu::new(Box::new(MockRenderer::new()), TvSystem::NTSC, Mirroring::VerticalMirroring, rom);
        // replace vram with mock
        ppu.vram = Box::new(MockMemory::new());
        ppu
    }

    #[test]
    fn write_to_0x2000_changes_control_register_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2000, 0x13);
        assert_eq!(0x13, ppu.registers.control);
    }

    #[test]
    fn write_to_0x2000_generates_nmi_if_nmi_bit_will_be_set_and_vblank_bit_is_set() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0x80;
        ppu.nmi_occured = false;
        ppu.write(0x2000, 0x83);
        assert_eq!(true, ppu.nmi_occured);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2000_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2000);
    }

    #[test]
    fn write_to_0x2001_changes_mask_register_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2001, 0x13);
        assert_eq!(0x13, ppu.registers.mask);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2001_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2001);
    }

    #[test]
    #[should_panic]
    fn write_to_0x2002_panics() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2002, 0xD4);
    }

    #[test]
    fn read_from_0x2002_returns_status_register_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0xD5;
        assert_eq!(0xD5, ppu.read(0x2002));
    }

    #[test]
    fn read_from_0x2002_clears_the_bit_7() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0xD5;
        ppu.read(0x2002);
        assert_eq!(0x55, ppu.registers.status);
    }

    #[test]
    fn read_from_0x2002_clears_the_address_latch() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.read(0x2002);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    fn read_from_0x2002_does_not_flip_the_address_latch_if_latch_is_clear() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.read(0x2002);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2003_changes_oam_address_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2003, 0x13);
        assert_eq!(0x13, ppu.registers.oam_address);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2003_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2003);
    }

    #[test]
    fn write_to_0x2004_changes_object_attribute_memory_at_oam_address() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_address = 0xFE;
        ppu.write(0x2004, 0x13);
        assert_eq!(0x13, ppu.object_attribute_memory[0xFE]);
    }

    #[test]
    fn write_to_0x2004_increments_oam_address_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_address = 0x21;
        ppu.write(0x2004, 0x34);
        assert_eq!(0x22, ppu.registers.oam_address);
    }

    #[test]
    fn read_from_0x2004_returns_oam_value_at_address() {
        let mut ppu = create_test_ppu();
        ppu.registers.oam_address = 0x56;
        ppu.object_attribute_memory[0x56] = 0x64;
        assert_eq!(0x64, ppu.read(0x2004));
    }

    #[test]
    fn first_write_to_0x2005_sets_fine_x_scroll_register_correctly() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.write(0x2005, 0x1B);
        assert_eq!(3, ppu.fine_x_scroll);
    }

    #[test]
    fn first_write_to_0x2005_sets_temporary_register_address_bits_correctly() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.registers.temporary = 0xFFFF;
        ppu.write(0x2005, 0x1B);
        assert_eq!(0xFFE3, ppu.registers.temporary);
    }

    #[test]
    fn second_write_to_0x2005_does_not_touch_fine_x_scroll() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.fine_x_scroll = 0;
        ppu.write(0x2005, 0x1B);
        assert_eq!(0, ppu.fine_x_scroll);
    }

    #[test]
    fn second_write_to_0x2005_sets_temporary_register_address_bits_correctly() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.registers.temporary = 0xFFFF;
        ppu.write(0x2005, 0x1B);
        assert_eq!(0x3C7F, ppu.registers.temporary);
    }



    #[test]
    #[should_panic]
    fn read_from_0x2005_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2005);
    }

    #[test]
    fn write_to_0x2005_sets_the_address_latch_if_it_was_unset_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.write(0x2005, 0x13);
        assert_eq!(true, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2005_unsets_the_address_latch_if_it_was_set_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.write(0x2005, 0x13);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    #[should_panic]
    fn read_from_0x2006_panics() {
        let mut ppu = create_test_ppu();
        ppu.read(0x2006);
    }

    #[test]
    fn write_to_0x2006_sets_the_address_latch_if_it_was_unset_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.write(0x2006, 0x13);
        assert_eq!(true, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2006_unsets_the_address_latch_if_it_was_set_before() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.write(0x2006, 0x13);
        assert_eq!(false, ppu.address_latch);
    }

    #[test]
    fn write_to_0x2006_writes_first_6_bits_of_high_byte_to_temporary_register_if_latch_is_unset() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = false;
        ppu.registers.temporary = 0x000;
        ppu.write(0x2006, 0xFF);
        assert_eq!(0x3F00, ppu.registers.temporary);
    }

    #[test]
    fn write_to_0x2006_writes_low_byte_to_temporary_register_if_latch_is_set() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.registers.temporary = 0x1234;
        ppu.write(0x2006, 0x6F);
        assert_eq!(0x126F, ppu.registers.temporary);
    }

    #[test]
    fn write_to_0x2006_copies_temporary_register_to_address_register_if_latch_is_set() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = true;
        ppu.registers.temporary = 0x1234;
        ppu.write(0x2006, 0x6F);
        assert_eq!(0x126F, ppu.vram_address);
    }

    #[test]
    fn write_to_0x2007_writes_to_memory_at_vram_address() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3435;
        ppu.write(0x2007, 0x13);
        assert_eq!(0x13, ppu.vram.read(0x3435));
    }

    #[test]
    fn read_from_0x2007_returns_vram_read_buffer_value_when_reading_below_0x3F00() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3EFF;
        ppu.vram_read_buffer = 0xA4;
        ppu.vram.write(0x3EFF, 0x23);
        assert_eq!(0xA4, ppu.read(0x2007));
    }

    #[test]
    fn read_from_0x2007_updates_read_buffer_to_value_at_current_address_when_reading_below_0x3F00() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3EFF;
        ppu.vram_read_buffer = 0xA4;
        ppu.vram.write(0x3EFF, 0xB9);
        ppu.read(0x2007);
        assert_eq!(0xB9, ppu.vram_read_buffer);
    }

    #[test]
    fn read_from_0x2007_returns_data_straight_from_vram_skipping_read_buffer_when_reading_at_or_above_0x3F00() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3F00;
        ppu.vram_read_buffer = 0xE1;
        ppu.vram.write(0x3F00, 0xBE);
        assert_eq!(0xBE, ppu.read(0x2007));
    }

    #[test]
    fn read_from_0x2007_sets_read_buffer_to_nametable_value_at_0x1000_before_address() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x3F00;
        ppu.vram_read_buffer = 0x14;
        ppu.vram.write(0x3F00, 0x200);
        ppu.vram.write(0x2F00, 0xB1);
        ppu.read(0x2007);
        assert_eq!(0xB1, ppu.vram_read_buffer);
    }

    #[test]
    fn read_from_0x2007_increments_vram_address_by_one_if_control_register_bit_2_is_0() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x00;
        ppu.vram_address = 0x2353;
        ppu.read(0x2007);
        assert_eq!(0x2354, ppu.vram_address);
    }

    #[test]
    fn read_from_0x2007_increments_vram_address_by_32_if_control_register_bit_2_is_1() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x04;
        ppu.vram_address = 0x2353;
        ppu.read(0x2007);
        assert_eq!(0x2353 + 32, ppu.vram_address);
    }

    #[test]
    fn write_to_0x2007_increments_vram_address_by_one_if_control_register_bit_2_is_0() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x00;
        ppu.vram_address = 0x2353;
        ppu.write(0x2007, 0x23);
        assert_eq!(0x2354, ppu.vram_address);
    }

    #[test]
    fn write_to_0x2007_increments_vram_address_by_32_if_control_register_bit_2_is_1() {
        let mut ppu = create_test_ppu();
        ppu.registers.control = 0x04;
        ppu.vram_address = 0x2353;
        ppu.write(0x2007, 0x23);
        assert_eq!(0x2353 + 32, ppu.vram_address);
    }

    #[test]
    fn write_to_address_0x2008_is_mirrored_to_control_register() {
        let mut ppu = create_test_ppu();
        ppu.write(0x2008, 0x13);
        assert_eq!(0x13, ppu.registers.control);
    }

    #[test]
    fn read_from_address_0x200A_is_mirrored_to_status_register() {
        let mut ppu = create_test_ppu();
        ppu.registers.status = 0xD5;
        assert_eq!(0xD5, ppu.read(0x200A));
    }

    #[test]
    fn write_to_address_between_0x2008_and_0x3FFF_is_mirrored_correctly() {
        let mut ppu = create_test_ppu();
        ppu.write(0x3013, 0x13);
        assert_eq!(0x13, ppu.registers.oam_address);
    }

    #[test]
    fn read_from_address_between_0x2008_and_0x3FFF_is_mirrored_correctly() {
        let mut ppu = create_test_ppu();
        ppu.object_attribute_memory[0xA9] = 0xF2;
        ppu.registers.oam_address = 0xA9;
        assert_eq!(0xF2, ppu.read(0x3014));
    }

    #[test]
    // write to data register -> write to vram at vram_address
    fn write_to_0x3FFF_writes_to_ppu_data_register() {
        let mut ppu = create_test_ppu();
        ppu.vram_address = 0x1234;
        ppu.write(0x3FFF, 0x13);
        assert_eq!(0x13, ppu.vram.read(0x1234));
    }

    #[test]
    fn read_from_0x3FFF_reads_from_ppu_data_register() {
        let mut ppu = create_test_ppu();
        ppu.vram_read_buffer = 0xF2;
        ppu.vram_address = 0x1234;
        assert_eq!(0xF2, ppu.read(0x3FFF));
    }

    #[test]
    #[should_panic]
    fn oam_dma_write_panics_if_data_length_is_not_256_bytes() {
        let data:Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut ppu = create_test_ppu();
        ppu.oam_dma_write(data);
    }

    #[test]
    fn oam_dma_write_replaces_oam() {
        let mut ppu = create_test_ppu();
        let mut data = vec![];
        for i in 0..256 {
            data.push(((i as u16) % 256) as u8);
        }
        ppu.oam_dma_write(data.clone());
        assert_eq!(data, ppu.object_attribute_memory);
    }

    #[test]
    fn get_cycle_count_returns_same_value_every_time_if_no_extra_cycles_are_needed() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 4;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 0;
        for _ in 0..100 {
            assert_eq!(4, ppu.get_cycle_count());
        }
    }

    #[test]
    fn get_cycle_count_does_not_modify_extra_cycle_counter_if_no_extra_cycles_are_needed() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 4;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 0;
        for _ in 0..100 {
            ppu.get_cycle_count();
            assert_eq!(0, ppu.tv_system.extra_cycle_counter);
        }
    }

    #[test]
    fn get_cycle_count_returns_extra_cycle_every_n_frames_if_cycle_is_required() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 3;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 5;
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
        assert_eq!(4, ppu.get_cycle_count());
        assert_eq!(3, ppu.get_cycle_count());
    }

    #[test]
    fn get_cycle_count_modifies_extra_cycle_counter_if_extra_cycle_is_needed() {
        let mut ppu = create_test_ppu();
        ppu.tv_system.ppu_cycles_per_cpu_cycle = 4;
        ppu.tv_system.ppu_extra_cycle_every_cpu_cycle = 7;
        for i in 1..100 {
            ppu.get_cycle_count();
            assert_eq!(i % 7, ppu.tv_system.extra_cycle_counter);
        }
    }

    #[test]
    fn ppu_sets_vblank_bit_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_set() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x80;
        ppu.registers.control = 0;
        ppu.execute_cycle();
        assert_eq!(0x80, ppu.registers.status & 0x80);
    }

    #[test]
    fn ppu_sets_vblank_bit_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_cleared() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0;
        ppu.registers.control = 0;
        ppu.execute_cycle();
        assert_eq!(0x80, ppu.registers.status & 0x80);
    }

    #[test]
    fn ppu_generates_nmi_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_set() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x00;
        ppu.registers.control = 0x80;
        ppu.execute_cycle();
        assert_eq!(true, ppu.nmi_occured);
    }

    #[test]
    fn ppu_does_not_generate_nmi_on_vblank_first_scanline_second_pixel_if_nmi_flag_is_cleared() {
        let mut ppu = create_test_ppu();
        ppu.current_scanline = 0;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x00;
        ppu.registers.control = 0x00;
        ppu.execute_cycle();
        assert_eq!(false, ppu.nmi_occured);
    }

    #[test]
    fn ppu_clears_vblank_bit_on_pre_render_scanline_second_pixel() {
        let mut ppu = create_test_ppu();

        ppu.tv_system.vblank_frames = 50;
        ppu.current_scanline = 50;
        ppu.pos_at_scanline = 1;
        ppu.registers.status = 0x80;
        ppu.registers.control = 0;

        ppu.execute_cycle();

        assert_eq!(0x00, ppu.registers.status);
    }

    #[test]
    fn nmi_occured_returns_true_if_nmi_has_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = true;
        assert_eq!(true, ppu.nmi_occured());
    }

    #[test]
    fn nmi_occured_returns_false_if_nmi_has_not_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = false;
        assert_eq!(false, ppu.nmi_occured());
    }
    #[test]
    fn nmi_occured_clears_nmi_status_if_nmi_has_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = true;
        ppu.nmi_occured();
        assert_eq!(false, ppu.nmi_occured);
    }

    #[test]
    fn nmi_does_nothing_to_nmi_status_nmi_has_not_occured() {
        let mut ppu = create_test_ppu();

        ppu.nmi_occured = false;
        ppu.nmi_occured();
        assert_eq!(false, ppu.nmi_occured);
    }

}
