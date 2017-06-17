extern crate sdl2;
use self::sdl2::render::{Canvas, TextureCreator};
use self::sdl2::video::{Window, WindowContext};

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
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

pub struct SDLRenderer<'a> {
    canvas: &'a mut Canvas<Window>,
    texture: sdl2::render::Texture<'a>,
}

impl<'a> SDLRenderer<'a> {
    pub fn new(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>) -> SDLRenderer<'a> {
        let texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB888, 256, 240).unwrap();

        SDLRenderer {
            canvas: canvas,
            texture: texture,
        }
    }
}

impl<'a> Renderer for SDLRenderer<'a> {
    fn render(&mut self, pixels: &Vec<Pixel>) {

        self.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in (0..240) {
                 for x in (0..256) {
                     let pixel = pixels[y * 256 + x].clone();
                     let offset = y*pitch + 4*x;
                     buffer[offset + 0] = pixel.b as u8;
                     buffer[offset + 1] = pixel.g as u8;
                     buffer[offset + 2] = pixel.r as u8;
                     buffer[offset + 3] = 255 as u8;
                 }
             }
         }).unwrap();

        self.canvas.clear();
        self.canvas.copy(&self.texture, None, Rect::new(0, 0, 256*2, 240*2));
        self.canvas.present();
    }
}
