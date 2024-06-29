#![feature(let_chains)]
#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

mod error;
mod gameboy;
mod lr35902;
mod memory;
mod renderer;
mod rhai_engine;
mod tests;
mod video;

use crate::gameboy::GameBoy;
use crate::renderer::Renderer;
use std::time::{Duration, Instant};

const TARGET_FPS: f64 = 59.73;
const TARGET_FRAME_DURATION: Duration = Duration::from_nanos((1_000_000_000.0 / TARGET_FPS) as u64);

fn main() {
    setup_logging();

    // Load the bootrom and cartridge, execute emulator
    let bootrom = include_bytes!("../external/roms/dmg_boot.bin").to_vec();
    let cartridge = include_bytes!("../external/roms/Tetris.gb").to_vec();

    let mut renderer = Renderer::new();
    let mut gb = GameBoy::new(bootrom, cartridge);

    loop {
        let throttle_timer = Instant::now();

        if !renderer.handle_events() {
            break;
        }

        gb.run_frame();
        renderer.update_texture(&gb.render_background());
        renderer.render();

        let frame_duration = throttle_timer.elapsed();
        if frame_duration < TARGET_FRAME_DURATION {
            spin_sleep::sleep(TARGET_FRAME_DURATION - frame_duration);
        }
    }
}

fn setup_logging() {
    #[cfg(debug_assertions)]
    {
        use fern::Dispatch;
        use log::LevelFilter;
        use std::fs::OpenOptions;

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
}
