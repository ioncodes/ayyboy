#![feature(let_chains)]

mod gameboy;
mod lr35902;
mod memory;
mod rhai_engine;
mod video;

use crate::gameboy::GameBoy;
use fern::Dispatch;
use log::LevelFilter;
use std::fs::OpenOptions;

fn main() {
    // Setup logger
    const LOG_PATH: &str = "trace.log";
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
    let bootrom = include_bytes!("../external/dmg_boot.bin").to_vec();
    let cartridge = include_bytes!("../external/Asterix (USA) (Proto 1).gb").to_vec();

    // let mut gb = GameBoy::with_rhai(bootrom, vec![0u8; cartridge.len()], "external/drm_patch.rhai".into());
    // gb.install_breakpoints(vec![0xe9]);

    let mut gb = GameBoy::new(bootrom, cartridge);

    loop {
        gb.tick();
    }
}
