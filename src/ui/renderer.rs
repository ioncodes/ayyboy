use crate::gameboy::GameBoy;
use crate::ui::debugger::Debugger;
use crate::ui::settings::Settings;
use crate::video::palette::{Color, Palette};
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::{vec2, CentralPanel, Color32, ColorImage, Context, Image, Key, TextureHandle, TextureOptions};
use eframe::{App, CreationContext, Frame};
use log::warn;
use std::time::{Duration, Instant};

const TARGET_FPS: f64 = 59.73;
const TARGET_FRAME_DURATION: Duration = Duration::from_nanos((1_000_000_000.0 / TARGET_FPS) as u64);

pub struct Renderer<'a> {
    debugger: Debugger,
    screen_texture: TextureHandle,
    gameboy: GameBoy<'a>,
    settings: Settings,
}

impl<'a> Renderer<'a> {
    pub fn new(cc: &CreationContext, gameboy: GameBoy<'a>, settings: Settings) -> Renderer<'a> {
        let screen_texture = cc.egui_ctx.load_texture(
            "screen_texture",
            ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        Renderer {
            debugger: Debugger::new(),
            screen_texture,
            gameboy,
            settings,
        }
    }

    pub fn update_screen(&mut self, palette_data: &[[Palette; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
        let mut pixels = vec![Color32::BLACK; SCREEN_WIDTH * SCREEN_HEIGHT];

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let color: Color = palette_data[y][x].into();
                pixels[y * SCREEN_WIDTH + x] = Color32::from_rgba_premultiplied(color[0], color[1], color[2], 255);
            }
        }

        let image = ColorImage {
            size: [SCREEN_WIDTH, SCREEN_HEIGHT],
            pixels,
        };

        self.screen_texture.set(image, TextureOptions::NEAREST);
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(Key::F1)) {
            self.debugger.toggle_window();
        }
    }
}

impl App for Renderer<'_> {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let throttle_timer = Instant::now();

        self.handle_input(ctx);

        self.gameboy.run_frame();
        self.update_screen(&self.gameboy.emulated_frame());

        CentralPanel::default().show(ctx, |ui| {
            let image = Image::new(&self.screen_texture);
            let image = image.fit_to_exact_size(vec2((SCREEN_WIDTH * 4) as f32, (SCREEN_WIDTH * 4) as f32));
            image.paint_at(ui, ui.ctx().screen_rect());
        });

        self.debugger.update_ui(ctx);

        ctx.request_repaint();

        if !self.settings.uncapped {
            let frame_duration = throttle_timer.elapsed();
            if frame_duration < TARGET_FRAME_DURATION {
                spin_sleep::sleep(TARGET_FRAME_DURATION - frame_duration);
            } else {
                warn!(
                    "Frame took too long: {:?} with a delta of {:?}",
                    frame_duration,
                    frame_duration - TARGET_FRAME_DURATION
                );
            }
        }
    }
}
