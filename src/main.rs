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
use clap::Parser;
use dark_light::Mode;
use eframe::egui::{Style, ViewportBuilder, Visuals};
use eframe::NativeOptions;
use fern::Dispatch;
use frontend::settings::Settings;
use log::{info, LevelFilter};
use std::fs::File;
use zip::ZipArchive;

#[derive(Parser, Debug)]
struct Args {
    rom: String,
    #[arg(long)]
    bios: Option<String>,
    #[arg(long, default_value_t = false)]
    log_to_file: bool,
}

fn main() {
    let args = Args::parse();

    setup_logging(args.log_to_file);

    let bootrom = match &args.bios {
        Some(bios) => Some(std::fs::read(bios).expect("Failed to read BIOS file")),
        None => None,
    };

    let mut gameboy = GameBoy::new(bootrom, load_rom(&args.rom));

    // if there's a sav file, load into cart
    let save_path = format!("{}.sav", &args.rom);
    if let Ok(cart_ram) = std::fs::read(&save_path) {
        gameboy.mmu.cartridge.load_ram(cart_ram);
        info!("Loaded cartridge RAM from {}", save_path);
    }

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([
                (SCREEN_WIDTH * SCALE) as f32,
                (SCREEN_HEIGHT * SCALE) as f32,
            ])
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
            Box::new(Renderer::new(cc, gameboy, Settings { rom_path: args.rom }))
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
    let base_config = if !log_to_file {
        Dispatch::new()
            .level(LevelFilter::Off)
            .level_for("ayyboy", LevelFilter::Debug)
            .chain(std::io::stdout())
    } else {
        const LOG_PATH: &str = "./ayyboy_trace.log";
        std::fs::remove_file(LOG_PATH).unwrap_or_default();

        Dispatch::new()
            .level(LevelFilter::Off)
            .level_for("ayyboy", LevelFilter::Trace)
            .chain(fern::log_file(LOG_PATH).unwrap())
    };

    base_config
        .format(move |out, message, record| {
            out.finish(format_args!("[{}] {}", record.level(), message))
        })
        .apply()
        .unwrap();
}
