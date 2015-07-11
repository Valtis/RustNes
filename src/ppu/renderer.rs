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
        let texture = renderer.create_texture_streaming(PixelFormatEnum::IYUV, (256, 224)).unwrap();
        Renderer {
            renderer: renderer,
            texture: texture,
        }
    }

    pub fn render(&mut self, pixels: Vec<Pixel>) {
        // currently just copy\paste from the SDL 2 rust examples. Will be changed in the future
        //println!("Render");
        self.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            // `pitch` is the width of the Y component
            // The U and V components are half the width and height of Y

            let w = 256;
            let h = 224;

            // Set Y (constant)
            for y in 0..h {
                for x in 0..w {
                    let offset = y*pitch + x;
                    buffer[offset] = 128;
                }
            }

            let y_size = pitch*h;

            // Set U and V (X and Y)
            for y in 0..h/2 {
                for x in 0..w/2 {
                    let u_offset = y_size + y*pitch/2 + x;
                    let v_offset = y_size + (pitch/2 * h/2) + y*pitch/2 + x;
                    buffer[u_offset] = (x*2) as u8;
                    buffer[v_offset] = (y*2) as u8;
                }
            }
        }).unwrap();

        self.renderer.clear();
        self.renderer.copy(&self.texture, None, Some(Rect::new_unwrap(100, 100, 256, 256)));
        self.renderer.present();
    }
}
