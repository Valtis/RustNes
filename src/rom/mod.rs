// see http://wiki.nesdev.com/w/index.php/INES for more information
use std::fs::File;
use std::io::Read;
use std::fmt;
use memory::Memory;

pub fn read_rom(file_path: &str) -> Rom {

    let mut rom = Rom::new();

    let mut rom_file = File::open(file_path).unwrap_or_else(|e| {
        panic!("Could not open the rom file {}: {}", file_path, e);
        });


    rom.read_header(&mut rom_file);
    rom.read_trainer_field(&mut rom_file);
    rom.read_prg_rom(&mut rom_file);
    rom.read_chr_rom(&mut rom_file);
    // mappers are currently unimplemented; mapper 0 is hardcoded
    // thus, panic if other mappers are used
    if rom.header.mapper != 0 {
        panic!("Currently only mapper 0 is implemented (rom uses mapper {})", rom.header.mapper);
    }
    rom
}


fn read_bytes_from_file_or_panic(length:u64, file: &mut File, err_msg: &str) -> Vec<u8>{
    let mut buf = vec![];
    let read_bytes =
        file
            .take(length)
            .read_to_end(&mut buf)
            .unwrap_or_else(
                |e| {
                    panic!("{}: {}", err_msg, e);
                });

    if read_bytes != buf.len() {
        panic!("{}: {} bytes read but {} was expected", err_msg, read_bytes, length);
    }
    buf
}

struct RamArray {
    data: [u8; 0x2000],
}

#[derive(Debug)]
pub struct Rom {
    pub header: RomHeader,
    trainer: Vec<u8>, // length is 0 if no trainer is present
    prg_rom_data: Vec<u8>,
    chr_rom_data: Vec<u8>,
    work_ram: RamArray,
}

impl fmt::Debug for RamArray {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.data[..].fmt(formatter)
    }
}



// this is an inefficient implementation, requiring address calculations on each read\write
fn get_offset_temp_hardcoded_impl(length: u8, address: u16) -> usize {

    // program rom is mapped to memory addresses 0x8000 - 0xBFFF and 0xC000 - 0xFFFF
    // if rom size is 16kb, 0x8000 - 0xBFFF and 0xC000 - 0xFFFF are mirrored
    // otherwise first 16kb of rom is mapped to 0x8000 -> and second 16kb is mapped to C0000 ->

    let base = address & 0x3FFF; // mask first two bits away to get offset
    if length > 1 && (address & 0xC000 == 0xC000) {
        (base + 0x4000) as usize // if rom size is not 16kb and address is 0xC000 ->, map to second 16kb
    } else {
        base as usize
    }
}

impl Memory for Rom {

    fn read(&mut self, address: u16) -> u8 {
        // TODO: Implement mappers & let them handle this
        // for now, mapper 0 is hardcoded (poorly)

        // basically the hardcoded assumption right now is that any read form 0x2000 or below is to chr rom,
        // otherwise it's to prg rom
        if address < 0x2000 {
            self.chr_rom_data[address as usize]
        } else if address >= 0x6000 && address <= 0x7FFF {
            // work ram on the rom
            self.work_ram.data[(address - 0x6000) as usize]
        } else {
            let offset = get_offset_temp_hardcoded_impl(self.header.prg_rom_size, address);
            self.prg_rom_data[offset]
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address >= 0x6000 && address <= 0x7FFF {
            // work ram on the rom
            self.work_ram.data[(address - 0x6000) as usize] = value;
        } else {
            panic!("Invalid write into rom memory address space (address 0x{:04X}, value: 0x{:04X})",
                address,
                value);
        }
    }
}



impl Rom {
    pub fn new() -> Rom {
        Rom {
            header: RomHeader::new(),
            trainer: vec![],
            prg_rom_data: vec![],
            chr_rom_data: vec![],
            work_ram: RamArray { data: [0; 0x2000] },

        }
    }

    fn read_header(&mut self, rom_file: &mut File) {
        RomHeader::verify_magic_number_or_panic(rom_file);
        self.header.read_prg_rom_size(rom_file);
        self.header.read_chr_rom_size(rom_file);
        self.header.read_flags_6(rom_file);
        self.header.read_flags_7(rom_file);
        self.header.read_prg_ram_size(rom_file);
        self.header.read_flags_9(rom_file);
        RomHeader::read_padding(rom_file);
    }

    fn read_trainer_field(&mut self, rom_file: &mut File) {
        // check if the trainer bit is set - if not, there is no trainer and do nothing
        if self.header.has_trainer {

            self.trainer = read_bytes_from_file_or_panic(512, rom_file,
                "Could not read the trainer field from the rom");
        }
    }

    fn read_prg_rom(&mut self, rom_file: &mut File) {
        let prg_rom_unit_size = 16384;
        let size = prg_rom_unit_size * self.header.prg_rom_size as u64;
        self.prg_rom_data = read_bytes_from_file_or_panic(size, rom_file,
            "Could not read prg rom data from rom");
    }

    fn read_chr_rom(&mut self, rom_file: &mut File) {
        let chr_rom_unit_size = 8192;
        let size = chr_rom_unit_size * self.header.chr_rom_size as u64;
        self.chr_rom_data = read_bytes_from_file_or_panic(size, rom_file,
            "Could not read chr rom data from rom");
    }
}

#[derive(Debug, Clone)]
pub enum TvSystem {
    Uninitialized,
    PAL,
    NTSC
}

#[derive(Debug, Clone)]
pub enum Mirroring {
    Uninitialized,
    HorizontalMirroring,
    VerticalMirroring,
    FourScreenVRAM
}


#[derive(Debug)]
pub struct RomHeader {
    pub prg_rom_size:u8, // size in 16kb units
    pub chr_rom_size:u8, // size in 8kb units - if 0, chr ram is used
    pub prg_ram_size:u8, // size in 8kb units - if 0, 8kb of ram is assumed
    pub mapper: u8,
    pub mirroring: Mirroring,
    pub tv_system: TvSystem,
    has_trainer: bool,
    has_battery_backing: bool,
}


impl RomHeader {
    fn new() -> RomHeader {
        RomHeader {
            prg_rom_size: 0,
            chr_rom_size:0,
            prg_ram_size:0,
            mapper: 0,
            mirroring: Mirroring::Uninitialized,
            tv_system: TvSystem::Uninitialized,
            has_trainer: false,
            has_battery_backing: false,
        }
    }

    fn verify_magic_number_or_panic(rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(4, rom_file,
            "Could not read the magic number from the header");

        if !(buf[0] == 0x4E && buf[1] == 0x45 && buf[2] == 0x53 && buf[3] == 0x1A) {
            panic!("Invalid magic number");
        }
    }

    fn read_prg_rom_size(&mut self, rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(1, rom_file,
            "Could not read the prg rom size from the header");
        self.prg_rom_size = buf[0];
    }

    fn read_chr_rom_size(&mut self, rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(1, rom_file,
            "Could not read the chr rom size from the header");
        self.chr_rom_size = buf[0];
    }



    /*
    Documentation on flags 6:

    76543210
    ||||||||
    ||||+||+- 0xx0: vertical arrangement/horizontal mirroring (CIRAM A10 = PPU A11)
    |||| ||   0xx1: horizontal arrangement/vertical mirroring (CIRAM A10 = PPU A10)
    |||| ||   1xxx: four-screen VRAM
    |||| |+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
    |||| +--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
    ++++----- Lower nybble of mapper number
*/
    fn read_flags_6(&mut self, rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(1, rom_file,
            "Could not read the flags_6 field from header");

        // if bit 2 is set, trainer is present
        self.has_trainer = (buf[0] & (1 << 2)) != 0;

        // if bit 1 is set, battery backed memory (or other persistent memory) is present
        self.has_battery_backing = (buf[0] & (1 << 1)) != 0;

        // if bit 3 is set, bit 0 is ignored
        if (buf[0] & (1 << 3)) != 0 {
            self.mirroring = Mirroring::FourScreenVRAM;
        } else { // otherwise, read bit 0
            if (buf[0] & 1) == 0 {
                self.mirroring = Mirroring::HorizontalMirroring;
            } else {
                self.mirroring = Mirroring::VerticalMirroring;
            }
        }

        // set lower 4 bits of mapper number
        let lower_nybble =  buf[0] >> 4;
        self.mapper = self.mapper & 0xf0; // set lower 4 bits to 0, in case they were not
        self.mapper = self.mapper | lower_nybble;
    }

    /*
    Documentation on flags 7:

    76543210
    ||||||||
    |||||||+- VS Unisystem
    ||||||+-- PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
    ||||++--- If equal to 2, flags 8-15 are in NES 2.0 format
    ++++----- Upper nybble of mapper number

    */
    fn read_flags_7(&mut self, rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(1, rom_file,
            "Could not read the flags_7 field from header");

        // check if nes 2.0 format; if so, panic as this is currently not supported
        if (buf[0] & 0x0C) >> 2 == 0x02 {
            panic!("Rom is in nes 2.0 format which is currently unsupported");
        }
        // extract the upper nybble of the mapper number
        let upper_nybble = 0xf0 & buf[0];
        // set upper nybble to zero, in case it wasn't
        self.mapper = self.mapper & 0x0f;
        self.mapper = self.mapper | upper_nybble;

        // unisystem - playchoice are currently ignored
    }

    fn read_prg_ram_size(&mut self, rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(1, rom_file,
            "Could not read the prg ram size from header");

        self.prg_ram_size = buf[0];
        // to quoth the documentation:
        // "Size of PRG RAM in 8 KB units (Value 0 infers 8 KB for compatibility; see PRG RAM circuit)"
        if self.prg_ram_size == 0 {
            self.prg_ram_size = 1;
        }
    }



    /*
        Documentation on flags 9:

        76543210
        ||||||||
        |||||||+- TV system (0: NTSC; 1: PAL)
        +++++++-- Reserved, set to zero

    */
    fn read_flags_9(&mut self, rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(1, rom_file,
            "Could not read the flags_9 field from header");

        // Bits 1 - 7 should be zero. Thus, if the value is greater than 1, one or more of these
        // bits are set and something is wrong (possibly unsupported ROM version)
        if buf[0] > 1 {
            panic!("flags_9 field has invalid value {}: Other bits than the first one are set",
            buf[0])
        }

        if buf[0] == 0 {
            self.tv_system = TvSystem::NTSC;
        } else {
            self.tv_system = TvSystem::PAL;
        }
    }

    fn read_padding(rom_file: &mut File) {
        let buf = read_bytes_from_file_or_panic(6, rom_file,
            "Could not read the padding from the header");

        if !(buf[0] == 0 && buf[1] == 0 && buf[2] == 0 && buf[3] == 0 && buf[4] == 0 && buf[5] == 0) {
            panic!("Invalid padding: Padding is expected to be zero initialized");
        }
    }
}
