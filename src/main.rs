use crate::gameboy::GameBoy;
use fern::Dispatch;
use log::LevelFilter;
use std::fs::OpenOptions;

mod gameboy;
mod lr35902;
mod memory;
mod video;

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
        .chain(file)
        //.chain(std::io::stdout())
        .format(move |out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .apply()
        .unwrap();

    // Load the bootrom and cartridge, execute emulator
    let bootrom = include_bytes!("../external/dmg_boot.bin").to_vec();
    let cartridge = include_bytes!("../external/Asterix (USA) (Proto 1).gb").to_vec();

    let mut gb = GameBoy::new(bootrom, cartridge);
    loop {
        gb.tick();
    }
}
