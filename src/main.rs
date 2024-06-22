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
use crate::video::ppu::{BACKGROUND_HEIGHT, BACKGROUND_WIDTH};
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use fern::Dispatch;
use log::LevelFilter;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::time::Duration;

fn main() {
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
    let cartridge = include_bytes!("../external/roms/Alleyway (World).gb").to_vec();

    // let mut gb = GameBoy::with_rhai(bootrom, vec![0u8; cartridge.len()], "external/drm_patch.rhai".into());
    // gb.install_breakpoints(vec![0xe9, 0xfa]);

    let mut gb = GameBoy::new(bootrom, cartridge);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("ayyboy", BACKGROUND_WIDTH as u32, BACKGROUND_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut bootfinished = false;
    let mut frame: Vec<(usize, usize, Color)> = Vec::new();

    'running: loop {
        for (x, y, color) in &frame {
            canvas.set_draw_color(*color);
            canvas.draw_point(Point::new(*x as i32, *y as i32)).unwrap();
        }

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

        if !bootfinished && let Some(background_map) = gb.tick() {
            bootfinished = true;

            for (i, tile) in background_map.iter().enumerate() {
                for (y, row) in tile.pixels.iter().enumerate() {
                    for (x, color) in row.iter().enumerate() {
                        let color = match color {
                            0b00 => Color::WHITE,
                            0b01 => Color::BLACK,
                            _ => panic!("Invalid color: {}", color),
                        };

                        frame.push((x + (i % 16) * 8, y + (i / 16) * 8, color));
                    }
                }
            }
        }

        canvas.present();

        //std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
