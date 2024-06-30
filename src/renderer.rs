use crate::video::palette::{Color, Palette};
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::video::Window;
use sdl2::EventPump;

pub struct Renderer {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    screen_texture: Texture,
}

impl Renderer {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("ayyboy", SCREEN_WIDTH as u32 * 4, SCREEN_HEIGHT as u32 * 4)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        let screen_texture = canvas
            .create_texture(
                PixelFormatEnum::RGB24,
                TextureAccess::Streaming,
                SCREEN_WIDTH as u32,
                SCREEN_HEIGHT as u32,
            )
            .unwrap();

        Self {
            canvas,
            event_pump,
            screen_texture,
        }
    }

    pub fn update_texture(&mut self, palette_data: &[[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
        self.screen_texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..SCREEN_HEIGHT {
                    for x in 0..SCREEN_WIDTH {
                        let color: Color = palette_data[y][x].into();
                        let offset = y * pitch + x * 3;
                        buffer[offset..offset + 3].copy_from_slice(&color);
                    }
                }
            })
            .unwrap();
    }

    pub fn render(&mut self) {
        self.canvas
            .copy(
                &self.screen_texture,
                None,
                Rect::new(0, 0, SCREEN_WIDTH as u32 * 4, SCREEN_HEIGHT as u32 * 4),
            )
            .unwrap();
        self.canvas.present();
    }

    pub fn handle_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return false,
                _ => {}
            }
        }
        true
    }
}
