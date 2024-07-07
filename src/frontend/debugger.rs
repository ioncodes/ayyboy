use eframe::egui;
use egui::Context;

pub struct Debugger {
    pub window_open: bool,
}

impl Debugger {
    pub fn new() -> Self {
        Self { window_open: false }
    }

    pub fn update_ui(&mut self, ctx: &Context) {
        egui::Window::new("Hello, egui!").open(&mut self.window_open).show(ctx, |ui| {
            ui.label("This example demonstrates using egui with pixels.");
            ui.label("Made with 💖 in San Francisco!");

            ui.separator();

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x /= 2.0;
                ui.label("Learn more about egui at");
                ui.hyperlink("https://docs.rs/egui");
            });
        });
    }

    pub fn toggle_window(&mut self) {
        self.window_open = !self.window_open;
    }
}
