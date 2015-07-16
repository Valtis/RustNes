use rom::TvSystem;

#[derive(Debug)]
pub struct TvSystemValues {
    pub tv_type: TvSystem,
    pub ppu_cycles_per_cpu_cycle: u8,
    // add extra ppu cycle for N cpu cycles. PAL requires extra cycle every 5 cycle
    // if 0, no extra cycles will be executed
    pub ppu_extra_cycle_every_cpu_cycle: u8,
    pub extra_cycle_counter: u8, // counter for above
    pub vblank_frames: u16,
    pub post_render_scanlines: u16,
}

impl TvSystemValues {
    pub fn new(tv_type: &TvSystem) -> TvSystemValues {
        match *tv_type {
            TvSystem::PAL => panic!("PAL support is not implemented"),
            TvSystem::NTSC => TvSystemValues {
                tv_type: tv_type.clone(),
                ppu_cycles_per_cpu_cycle: 3,
                ppu_extra_cycle_every_cpu_cycle: 0,
                extra_cycle_counter: 0,
                vblank_frames: 20,
                post_render_scanlines: 1,
            },
            _ => panic!("Invalid TV system type given for ppu: {:?}", tv_type),
        }

    }
}
