#![feature(let_chains)]
#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

mod error;
mod gameboy;
mod lr35902;
mod memory;
mod rhai_engine;
mod tests;
mod ui;
mod video;

use crate::gameboy::GameBoy;
use crate::ui::renderer::Renderer;
use crate::ui::settings::Settings;
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::{FontFamily, FontId, Style, TextStyle, ViewportBuilder, Visuals};
use eframe::NativeOptions;

fn main() {
    setup_logging();

    let args: Vec<String> = std::env::args().collect();
    let bootrom = include_bytes!("../external/roms/dmg_boot.bin").to_vec();
    let cartridge = std::fs::read(&args[1]).expect("Failed to read ROM file");
    let gameboy = GameBoy::new(bootrom, cartridge);
    let uncapped = args.iter().any(|arg| arg == "--uncapped");

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([(SCREEN_WIDTH * 4) as f32, (SCREEN_HEIGHT * 4) as f32])
            .with_resizable(true),
        vsync: false,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "ayyboyy",
        native_options,
        Box::new(move |cc| {
            let style = Style {
                visuals: Visuals::light(),
                text_styles: [
                    (TextStyle::Body, FontId::new(14.0, FontFamily::Monospace)),
                    (TextStyle::Button, FontId::new(14.0, FontFamily::Monospace)),
                    (TextStyle::Heading, FontId::new(16.0, FontFamily::Monospace)),
                    (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
                ]
                .into(),
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(Renderer::new(cc, gameboy, Settings { uncapped }))
        }),
    );
}

fn setup_logging() {
    use fern::Dispatch;
    use log::LevelFilter;

    // Setup logger
    const LOG_PATH: &str = "./external/ayyboy_trace.log";
    std::fs::remove_file(LOG_PATH).unwrap_or_default();

    let mut base_config = Dispatch::new()
        .level(LevelFilter::Trace)
        .chain(Dispatch::new().level(LevelFilter::Info).chain(std::io::stdout()))
        .format(move |out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)));

    #[cfg(debug_assertions)]
    {
        base_config = base_config.chain(
            Dispatch::new()
                .level(LevelFilter::Trace)
                .chain(fern::log_file(LOG_PATH).unwrap()),
        );
    }

    base_config.apply().unwrap();
}
