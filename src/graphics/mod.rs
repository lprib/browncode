use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;

//in pixels (defined by PIX_SIZE)
const SCREEN_WIDTH: u32 = 128;
const SCREEN_HEIGHT: u32 = 128;
const PIX_SIZE: u32 = 8;

pub struct Graphics {
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl Graphics {
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

        Ok(Graphics { canvas, event_pump })
    }
}
