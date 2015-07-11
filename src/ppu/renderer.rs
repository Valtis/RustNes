extern crate sdl2;
use self::sdl2::pixels::PixelFormatEnum;
use self::sdl2::rect::Rect;
use self::sdl2::keyboard::Keycode;

pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl Pixel {
    fn new(r: u8, g: u8, b: u8) -> Pixel {
        Pixel {
            r: r,
            g: g,
            b: b,
        }
    }
}

pub struct Renderer<'a> {
    renderer: sdl2::render::Renderer<'a>,
    texture: sdl2::render::Texture,
}


impl<'a> Renderer<'a> {
    pub fn new(renderer: sdl2::render::Renderer<'a>) -> Renderer<'a> {
        let texture = renderer.create_texture_streaming(PixelFormatEnum::RGB888, (256*2, 224*2)).unwrap();
        Renderer {
            renderer: renderer,
            texture: texture,
        }
    }

    pub fn render(&mut self, pixels: Vec<Pixel>) {
        // currently just copy\paste from the SDL 2 rust examples. Will be changed in the future
        self.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {

            for y in (0..224*2) {
                 for x in (0..256*2) {
                     let offset = y*pitch + 4*x;
                     buffer[offset + 0] = 255 as u8;
                     buffer[offset + 1] = 0 as u8;
                     buffer[offset + 2] = 0 as u8;
                     buffer[offset + 3] = 255 as u8;
                 }
             }
         }).unwrap();

        self.renderer.clear();
        self.renderer.copy(&self.texture, None, Some(Rect::new_unwrap(100, 100, 256*2, 224*2)));
        self.renderer.present();
    }
}
