extern crate sdl2;
use std::fmt;
use self::sdl2::pixels::PixelFormatEnum;
use self::sdl2::rect::Rect;

#[derive(Clone, Debug)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8) -> Pixel {
        Pixel {
            r: r,
            g: g,
            b: b,
        }
    }
}

pub trait Renderer {
    fn render(&mut self, pixels: &Vec<Pixel>);
}


impl fmt::Debug for Renderer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


pub struct SDLRenderer<'a> {
    renderer: sdl2::render::Renderer<'a>,
    texture: sdl2::render::Texture,
}

impl<'a> SDLRenderer<'a> {
    pub fn new(renderer: sdl2::render::Renderer<'a>) -> SDLRenderer<'a> {
        let texture = renderer.create_texture_streaming(PixelFormatEnum::RGB888, (256, 240)).unwrap();
        SDLRenderer {
            renderer: renderer,
            texture: texture,
        }
    }
}

impl<'a> Renderer for SDLRenderer<'a> {
    fn render(&mut self, pixels: &Vec<Pixel>) {
        self.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {

            for y in 0..240 {
                 for x in 0..256 {
                     let pixel = pixels[y * 256 + x].clone();
                     let offset = y*pitch + 4*x;
                     buffer[offset + 0] = pixel.b as u8;
                     buffer[offset + 1] = pixel.g as u8;
                     buffer[offset + 2] = pixel.r as u8;
                     buffer[offset + 3] = 255 as u8;
                 }
             }
         }).unwrap();

        self.renderer.clear();
        self.renderer.copy(&self.texture, None, Some(Rect::new_unwrap(0, 0, 256*2, 240*2)));
        self.renderer.present();
    }
}
