//! A simple graphics API using SDL2

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator, TextureQuery};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, TimerSubsystem};

use crate::util::append_u32;

//in pixels (defined by PIX_SIZE)
const SCREEN_WIDTH: u32 = 96;
const SCREEN_HEIGHT: u32 = 64;

const PIX_SIZE: u32 = 8;

const PIXEL_FORMAT: PixelFormatEnum = PixelFormatEnum::RGBA8888;

/// handles drawing to the screen and imput events
pub struct Graphics {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    timer: TimerSubsystem,
}

/// Creates sprites, which are stored in the Sprites struct
pub struct SpriteCreator(TextureCreator<WindowContext>);

/// Holds sprites, which can be used by Graphics.
/// This is separate from Graphics and SpriteCreator due to lifetime issues.
pub struct Sprites<'a> {
    sprite_creator: &'a SpriteCreator,
    textures: Vec<Texture<'a>>,
}

impl Graphics {
    /// Attempts to make a new graphics object.
    /// Calling this will create the window and display it
    pub fn try_new() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let window = video
            .window("Title", SCREEN_WIDTH * PIX_SIZE, SCREEN_HEIGHT * PIX_SIZE)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_scale(PIX_SIZE as f32, PIX_SIZE as f32)?;
        let event_pump = sdl.event_pump()?;
        let timer = sdl.timer()?;

        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color((0, 0, 0, 0));
        canvas.clear();

        Ok(Graphics {
            canvas,
            event_pump,
            timer,
        })
    }

    /// Write backbuffer to screen
    pub fn present(&mut self) {
        self.canvas.present();
    }

    /// Color is in RGBA8888 format
    pub fn draw_color(&mut self, color: u32) {
        self.canvas.set_draw_color(to_color(color));
    }

    pub fn pixel(&mut self, x: u32, y: u32) {
        self.canvas.draw_point((x as i32, y as i32)).unwrap();
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.canvas
            .fill_rect(Rect::new(x as i32, y as i32, w, h))
            .unwrap();
    }

    pub fn poll_exit(&mut self) {
        for e in self.event_pump.poll_iter() {
            if let Event::Quit { .. } = e {
                std::process::exit(0);
            }
        }
    }

    pub fn delay(&mut self, ms: u32) {
        self.timer.delay(ms);
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub fn is_key_pressed(&self, scancode: u32) -> bool {
        self.event_pump
            .keyboard_state()
            .is_scancode_pressed(Scancode::from_i32(scancode as i32).expect("not a valid scancode"))
    }

    pub fn sprite(&mut self, sprites: &Sprites, sprite_index: u32, x: u32, y: u32) {
        let tex = &sprites.textures[sprite_index as usize];
        let TextureQuery { width, height, .. } = tex.query();
        self.canvas
            .copy(tex, None, Rect::new(x as i32, y as i32, width, height))
            .unwrap();
    }

    pub fn get_sprite_creator(&self) -> SpriteCreator {
        SpriteCreator(self.canvas.texture_creator())
    }

    /// Uses Bresenham's line algorithm
    pub fn line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32) {
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let step_x = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let step_y = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.pixel(x0 as u32, y0 as u32);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let error_2 = 2 * error;
            if error_2 >= dy {
                error += dy;
                x0 += step_x
            }
            if error_2 <= dx {
                error += dx;
                y0 += step_y;
            }
        }
    }
}

impl<'a> Sprites<'a> {
    pub fn new(sprite_creator: &'a SpriteCreator) -> Self {
        Sprites {
            sprite_creator,
            textures: Vec::new(),
        }
    }

    /// Create a monochromatic sprite, where each bit in data represents a pixel.
    /// Returns the index of the sprite
    /// (which can be used to identify the sprite when using it in Graphics).
    /// w MUST be a multiple of 8
    pub fn create_sprite_mono(&mut self, data: &[u8], w: u32, h: u32, color: u32) -> u32 {
        let mut tex = self
            .sprite_creator
            .0
            .create_texture_static(PIXEL_FORMAT, w, h)
            .unwrap();
        tex.set_blend_mode(BlendMode::Blend);
        let mut new_tex_data = Vec::new();
        for byte in data {
            for bit_index in (0..8).rev() {
                if (byte >> bit_index) & 1u8 != 0u8 {
                    new_tex_data.push(color as u8);
                    new_tex_data.push((color >> 8) as u8);
                    new_tex_data.push((color >> 16) as u8);
                    new_tex_data.push((color >> 24) as u8);
                } else {
                    append_u32(&mut new_tex_data, 0);
                }
            }
        }
        // the pitch (in bytes) is w * 4 because there are 4 bytes (RGBA8888) per pixel
        tex.update(None, &new_tex_data, (w * 4) as usize).unwrap();
        self.textures.push(tex);

        (self.textures.len() - 1) as u32
    }
}

/// Formates the u32 based on PIXEL_FORMAT const
fn to_color(n: u32) -> Color {
    let pixel_format =
        unsafe { PixelFormat::from_ll(sdl2::sys::SDL_AllocFormat(PIXEL_FORMAT as u32)) };
    Color::from_u32(&pixel_format, n)
}
