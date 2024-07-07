use crate::frontend::debugger::Debugger;
use crate::gameboy::GameBoy;
use crate::video::palette::{Color, Palette};
use crate::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::{vec2, Align2, CentralPanel, Color32, ColorImage, Context, Image, Key, TextureHandle, TextureOptions, Window};
use eframe::{App, CreationContext, Frame};

pub const SCALE: usize = 4;

pub struct Renderer {
    debugger: Debugger,
    screen_texture: TextureHandle,
    gb: GameBoy,
    running: bool,
}

impl Renderer {
    pub fn new(cc: &CreationContext, gameboy: GameBoy) -> Renderer {
        let screen_texture = cc.egui_ctx.load_texture(
            "screen_texture",
            ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        Renderer {
            debugger: Debugger::new(),
            screen_texture,
            gb: gameboy,
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
        } else if !self.running && !self.debugger.window_open {
            Window::new("Controls")
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Arrow keys to move");
                    ui.label("A and S to interact");
                    ui.label("Enter to start");
                    ui.label("Backspace to select");
                    ui.separator();
                    ui.label("Press Space to start/stop emulation");
                    ui.label("Press F1 to open debugger");
                });
        }

        CentralPanel::default().show(ctx, |ui| {
            let image = Image::new(&self.screen_texture);
            let image = image.fit_to_exact_size(vec2((SCREEN_WIDTH * SCALE) as f32, (SCREEN_WIDTH * SCALE) as f32));
            image.paint_at(ui, ui.ctx().screen_rect());
        });

        self.debugger.update_ui(ctx);

        ctx.request_repaint();
    }
}
