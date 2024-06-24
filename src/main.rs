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
use crate::video::palette::Palette;
use crate::video::ppu::{BACKGROUND_HEIGHT, BACKGROUND_WIDTH};
use crate::video::tile::Tile;
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use fern::Dispatch;
use log::LevelFilter;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::video::Window;
use sdl2::EventPump;
use std::fs::OpenOptions;
use std::time::Duration;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
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
        //.chain(file)
        //.chain(std::io::stdout())
        .format(move |out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .apply()
        .unwrap();

    // Load the bootrom and cartridge, execute emulator
    let bootrom = include_bytes!("../external/roms/dmg_boot.bin").to_vec();
    let cartridge = include_bytes!("../external/roms/dmg-acid2.gb").to_vec();

    let (mut canvas, mut event_pump) = setup_renderer();

    let mut tilemap_texture = canvas
        .create_texture(
            PixelFormatEnum::RGB24,
            TextureAccess::Streaming,
            // BACKGROUND_WIDTH as u32,
            // BACKGROUND_HEIGHT as u32,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )
        .unwrap();

    //let tilemap: Vec<Tile> = Vec::new();
    //let (tx, rx) = watch::channel(tilemap.clone());

    let (tx, rx) = watch::channel([[Palette::default(); SCREEN_WIDTH]; SCREEN_HEIGHT]);

    tokio::spawn(async move {
        let mut gb = GameBoy::new(bootrom, cartridge);
        // let mut gb = GameBoy::with_rhai(bootrom, vec![0u8; cartridge.len()], "external/drm_patch.rhai".into());
        // gb.install_breakpoints(vec![0xe9, 0xfa]);

        loop {
            gb.tick();

            if gb.ready_to_render() {
                // TODO: THis was render_tilemap
                //tx.send(gb.render_tilemap()).unwrap();
                //tx.send(gb.render_backgroundmap()).unwrap();
                tx.send(gb.render_background()).unwrap();
            }
        }
    });

    //reset_tilemap_color(&mut tilemap_texture);

    'running: loop {
        canvas.copy(&tilemap_texture, None, None).unwrap();

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

        let tilemap = rx.borrow().clone();
        //(&tilemap, 32, &mut tilemap_texture); // 16 for tilemap, 32 for backgroundmap
        update_texture_ex(&tilemap, &mut tilemap_texture);
        canvas.present();

        std::thread::sleep(Duration::new(0, 1_000_000_000 / 60));
    }
}

fn setup_renderer() -> (Canvas<Window>, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        //.window("ayyboy", BACKGROUND_WIDTH as u32, BACKGROUND_HEIGHT as u32)
        .window("ayyboy", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    (canvas, event_pump)
}

fn reset_tilemap_color(tilemap_texture: &mut Texture) {
    let white_rgb: Color = Palette::White.into();
    tilemap_texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..BACKGROUND_HEIGHT {
                for x in 0..BACKGROUND_WIDTH {
                    let offset = y * pitch + x * 3;
                    buffer[offset..offset + 3].copy_from_slice(&[white_rgb.r, white_rgb.g, white_rgb.b]);
                }
            }
        })
        .unwrap();
}

fn update_texture(tilemap: &Vec<Tile>, pitch: usize, tilemap_texture: &mut Texture) {
    for (i, tile) in tilemap.iter().enumerate() {
        for y in 0..8 {
            for x in 0..8 {
                let color: Color = tile.pixels[y][x].into();
                let rgb = [color.r, color.g, color.b];

                let x = (x + (i % pitch) * 8) as i32;
                let y = (y + (i / pitch) * 8) as i32;

                tilemap_texture.update(Rect::new(x, y, 1, 1), &rgb, 3 * 8).unwrap();
            }
        }
    }
}

fn update_texture_ex(palette_data: &[[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT], texture: &mut Texture) {
    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..SCREEN_HEIGHT {
                for x in 0..SCREEN_WIDTH {
                    let color: Color = palette_data[y][x].into();
                    let offset = y * pitch + x * 3;
                    buffer[offset..offset + 3].copy_from_slice(&[color.r, color.g, color.b]);
                }
            }
        })
        .unwrap();
}
