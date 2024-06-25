#![feature(let_chains)]
#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

mod gameboy;
mod lr35902;
mod memory;
mod rhai_engine;
mod tests;
mod video;

use crate::gameboy::GameBoy;
use crate::video::palette::{Color, Palette};
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use fern::Dispatch;
use log::LevelFilter;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::video::Window;
use sdl2::EventPump;
use std::fs::OpenOptions;
use std::time::{Duration, Instant};

const TARGET_FPS: f64 = 59.73;
const TARGET_FRAME_DURATION: Duration = Duration::from_nanos((1_000_000_000.0 / TARGET_FPS) as u64);

fn main() {
    #[cfg(debug_assertions)]
    setup_logging();

    // Load the bootrom and cartridge, execute emulator
    let bootrom = include_bytes!("../external/roms/dmg_boot.bin").to_vec();
    let cartridge = include_bytes!("../external/roms/dmg-acid2.gb").to_vec();

    let (mut canvas, mut event_pump) = setup_renderer();

    let mut tilemap_texture = canvas
        .create_texture(
            PixelFormatEnum::RGB24,
            TextureAccess::Streaming,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )
        .unwrap();

    let mut gb = GameBoy::new(bootrom, cartridge);

    'running: loop {
        let throttle_timer = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        gb.run_frame();
        update_texture(&gb.render_background(), &mut tilemap_texture);
        canvas.copy(&tilemap_texture, None, None).unwrap();
        canvas.present();

        let frame_duration = throttle_timer.elapsed();
        if frame_duration < TARGET_FRAME_DURATION {
            spin_sleep::sleep(TARGET_FRAME_DURATION - frame_duration);
        }
    }

    unsafe {
        tilemap_texture.destroy();
    }
}

fn setup_logging() {
    // Setup logger
    const LOG_PATH: &str = "./external/ayyboy_trace.log";
    std::fs::remove_file(LOG_PATH).unwrap_or_default();

    let file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open(LOG_PATH)
        .unwrap();
    let _dispatch = Dispatch::new()
        .level(LevelFilter::Trace)
        .chain(file)
        //.chain(std::io::stdout())
        .format(move |out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .apply()
        .unwrap();
}

fn setup_renderer() -> (Canvas<Window>, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("ayyboy", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    (canvas, event_pump)
}

fn update_texture(palette_data: &[[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT], texture: &mut Texture) {
    texture
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
