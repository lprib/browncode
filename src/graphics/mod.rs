//! A simple graphics API using SDL2

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, TimerSubsystem};


//in pixels (defined by PIX_SIZE)
const SCREEN_WIDTH: u32 = 96;
const SCREEN_HEIGHT: u32 = 64;

const PIX_SIZE: u32 = 8;

const PIXEL_FORMAT: PixelFormatEnum = PixelFormatEnum::RGBA8888;

pub struct Graphics {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    timer: TimerSubsystem,
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
    pub fn set_draw_color(&mut self, color: u32) {
        let pixel_format =
            unsafe { PixelFormat::from_ll(sdl2::sys::SDL_AllocFormat(PIXEL_FORMAT as u32)) };
        self.canvas
            .set_draw_color(Color::from_u32(&pixel_format, color));
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32) {
        self.canvas.draw_point((x as i32, y as i32)).unwrap();
    }

    pub fn poll_events(&mut self) {
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
}
