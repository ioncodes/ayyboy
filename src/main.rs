#![feature(let_chains)]
#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

mod error;
mod frontend;
mod gameboy;
mod joypad;
mod lr35902;
mod memory;
mod sound;
mod tests;
mod video;

use crate::frontend::renderer::{Renderer, SCALE};
use crate::gameboy::GameBoy;
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use dark_light::Mode;
use eframe::egui::{Style, ViewportBuilder, Visuals};
use eframe::NativeOptions;
use fern::Dispatch;
use frontend::settings::Settings;
use log::{info, LevelFilter};
use std::fs::File;
use zip::ZipArchive;

const BOOTROM: &[u8] = include_bytes!("../external/roms/dmg_boot.bin");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let log_to_file = args.iter().any(|arg| arg == "--log-to-file");

    setup_logging(log_to_file);

    let filepath = args.get(1).expect("No ROM file provided").to_owned();
    let mut gameboy = GameBoy::new(BOOTROM.to_vec(), load_rom(&filepath));

    // if there's a sav file, load into cart
    let save_path = format!("{}.sav", &filepath);
    if let Ok(cart_ram) = std::fs::read(&save_path) {
        gameboy.mmu.cartridge.load_ram(cart_ram);
        info!("Loaded cartridge RAM from {}", save_path);
    }

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([(SCREEN_WIDTH * SCALE) as f32, (SCREEN_HEIGHT * SCALE) as f32])
            .with_resizable(true),
        vsync: false,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "ayyboyy",
        native_options,
        Box::new(move |cc| {
            let style = Style {
                visuals: match dark_light::detect() {
                    Mode::Dark => Visuals::dark(),
                    Mode::Light => Visuals::light(),
                    Mode::Default => Visuals::dark(),
                },
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(Renderer::new(
                cc,
                gameboy,
                Settings {
                    rom_path: filepath.clone(),
                },
            ))
        }),
    );
}

fn load_rom(filepath: &str) -> Vec<u8> {
    if filepath.ends_with(".zip") {
        let file = File::open(&filepath).unwrap();
        let unzipped_filepath = unzip_rom(file);
        info!("Unzipped {} to {}", &filepath, unzipped_filepath);
        std::fs::read(&unzipped_filepath).expect("Failed to read ROM file")
    } else {
        std::fs::read(&filepath).expect("Failed to read ROM file")
    }
}

fn unzip_rom(file: File) -> String {
    let mut archive = ZipArchive::new(file).unwrap();
    let mut rom = archive.by_index(0).unwrap();

    let filepath = match rom.enclosed_name() {
        Some(name) => name.to_owned(),
        None => panic!("No file found in zip archive"),
    };
    let tempfolder = std::env::temp_dir();
    let filepath = tempfolder.join(&filepath);
    let filepath = filepath.to_str().unwrap().to_owned();

    let mut unpacked_file = File::create(&filepath).unwrap();
    std::io::copy(&mut rom, &mut unpacked_file).unwrap();

    filepath
}

fn setup_logging(log_to_file: bool) {
    // Setup logger
    const LOG_PATH: &str = "./ayyboy_trace.log";
    std::fs::remove_file(LOG_PATH).unwrap_or_default();

    let mut base_config = Dispatch::new()
        .level(LevelFilter::Trace)
        .chain(Dispatch::new().level(LevelFilter::Info).chain(std::io::stdout()))
        .format(move |out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)));

    if log_to_file {
        base_config = base_config.chain(
            Dispatch::new()
                .level(LevelFilter::Trace)
                .chain(fern::log_file(LOG_PATH).unwrap()),
        );
    }

    base_config.apply().unwrap();
}
