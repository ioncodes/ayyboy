use crate::frontend::debugger::Debugger;
use crate::frontend::settings::Settings;
use crate::gameboy::GameBoy;
use crate::video::palette::{Color, Palette};
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::{vec2, CentralPanel, Color32, ColorImage, Context, Image, Key, TextureHandle, TextureOptions};
use eframe::{App, CreationContext, Frame};
use log::warn;
use std::time::{Duration, Instant};

const TARGET_FPS: f64 = 59.73;
const TARGET_FRAME_DURATION: Duration = Duration::from_nanos((1_000_000_000.0 / TARGET_FPS) as u64);
pub const SCALE: usize = 4;

pub struct Renderer {
    debugger: Debugger,
    screen_texture: TextureHandle,
    gb: GameBoy,
    settings: Settings,
    throttle_timer: Instant,
    running: bool,
}

impl Renderer {
    pub fn new(cc: &CreationContext, gameboy: GameBoy, settings: Settings) -> Renderer {
        let screen_texture = cc.egui_ctx.load_texture(
            "screen_texture",
            ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        Renderer {
            debugger: Debugger::new(),
            screen_texture,
            gb: gameboy,
            settings,
            throttle_timer: Instant::now(),
            running: false,
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

        ctx.input(|i| {
            if i.key_pressed(Key::Space) {
                self.running = !self.running;
            }

            if i.key_down(Key::Enter) {
                self.gb.mmu.joypad.update_button(Key::Enter, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::Enter, false);
            }

            if i.key_down(Key::Backspace) {
                self.gb.mmu.joypad.update_button(Key::Backspace, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::Backspace, false);
            }

            if i.key_down(Key::A) {
                self.gb.mmu.joypad.update_button(Key::A, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::A, false);
            }

            if i.key_down(Key::S) {
                self.gb.mmu.joypad.update_button(Key::S, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::S, false);
            }

            if i.key_down(Key::ArrowUp) {
                self.gb.mmu.joypad.update_button(Key::ArrowUp, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::ArrowUp, false);
            }

            if i.key_down(Key::ArrowDown) {
                self.gb.mmu.joypad.update_button(Key::ArrowDown, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::ArrowDown, false);
            }

            if i.key_down(Key::ArrowLeft) {
                self.gb.mmu.joypad.update_button(Key::ArrowLeft, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::ArrowLeft, false);
            }

            if i.key_down(Key::ArrowRight) {
                self.gb.mmu.joypad.update_button(Key::ArrowRight, true);
            } else {
                self.gb.mmu.joypad.update_button(Key::ArrowRight, false);
            }
        });
    }
}

impl App for Renderer {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.handle_input(ctx);

        if self.running {
            self.gb.run_frame();
            self.update_screen(&self.gb.ppu.pull_frame());
        }

        CentralPanel::default().show(ctx, |ui| {
            let image = Image::new(&self.screen_texture);
            let image = image.fit_to_exact_size(vec2((SCREEN_WIDTH * SCALE) as f32, (SCREEN_WIDTH * SCALE) as f32));
            image.paint_at(ui, ui.ctx().screen_rect());
        });

        self.debugger.update_ui(ctx);

        ctx.request_repaint();

        if !self.settings.uncapped {
            let frame_duration = self.throttle_timer.elapsed();

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

        self.throttle_timer = Instant::now();
    }
}
