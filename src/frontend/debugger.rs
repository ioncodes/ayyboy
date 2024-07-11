use eframe::egui::{self, vec2, Color32, ColorImage, Image, TextureHandle, TextureOptions, Window};
use egui::Context;

use crate::gameboy::GameBoy;
use crate::video::palette::Color;
use crate::video::tile::Tile;
use crate::video::{BACKGROUND_HEIGHT, BACKGROUND_WIDTH, TILESET_HEIGHT, TILESET_WIDTH};

use super::renderer::SCALE;

pub struct Debugger {
    pub window_open: bool,
    tileset_texture: TextureHandle,
    backgroundmap_texture: TextureHandle,
    windowmap_texture: TextureHandle,
}

impl Debugger {
    pub fn new(ctx: &Context) -> Self {
        let tileset_texture = ctx.load_texture(
            "tileset_texture",
            ColorImage::new([TILESET_WIDTH, TILESET_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        let backgroundmap_texture = ctx.load_texture(
            "backgroundmap_texture",
            ColorImage::new([BACKGROUND_WIDTH, BACKGROUND_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        let windowmap_texture = ctx.load_texture(
            "windowmap_texture",
            ColorImage::new([BACKGROUND_WIDTH, BACKGROUND_HEIGHT], Color32::BLACK),
            TextureOptions::NEAREST,
        );

        Self {
            window_open: false,
            tileset_texture,
            backgroundmap_texture,
            windowmap_texture,
        }
    }

    pub fn update_ui(&mut self, ctx: &Context, gb: &mut GameBoy) {
        if !self.window_open {
            return;
        }

        Window::new("Tileset").resizable(false).show(ctx, |ui| {
            let tileset = gb.dbg_render_tileset();
            Debugger::render_into_texture(
                &tileset,
                &mut self.tileset_texture,
                16,
                TILESET_WIDTH,
                TILESET_HEIGHT,
            );

            let image = Image::new(&self.tileset_texture);
            let image = image.fit_to_exact_size(vec2(
                (TILESET_WIDTH * (SCALE / 4)) as f32,
                (TILESET_HEIGHT * (SCALE / 4)) as f32,
            ));
            ui.add(image);
        });

        Window::new("Background Tilemap")
            .resizable(false)
            .show(ctx, |ui| {
                let backgroundmap = gb.dbg_render_background_tilemap();
                Debugger::render_into_texture(
                    &backgroundmap,
                    &mut self.backgroundmap_texture,
                    32,
                    BACKGROUND_WIDTH,
                    BACKGROUND_HEIGHT,
                );

                let image = Image::new(&self.backgroundmap_texture);
                let image = image.fit_to_exact_size(vec2(
                    (BACKGROUND_WIDTH * (SCALE / 4)) as f32,
                    (BACKGROUND_HEIGHT * (SCALE / 4)) as f32,
                ));
                ui.add(image);
            });

        Window::new("Window Tilemap")
            .resizable(false)
            .show(ctx, |ui| {
                let windowmap = gb.dbg_render_window_tilemap();
                Debugger::render_into_texture(
                    &windowmap,
                    &mut self.windowmap_texture,
                    32,
                    BACKGROUND_WIDTH,
                    BACKGROUND_HEIGHT,
                );

                let image = Image::new(&self.windowmap_texture);
                let image = image.fit_to_exact_size(vec2(
                    (BACKGROUND_WIDTH * (SCALE / 4)) as f32,
                    (BACKGROUND_HEIGHT * (SCALE / 4)) as f32,
                ));
                ui.add(image);
            });
    }

    pub fn toggle_window(&mut self) {
        self.window_open = !self.window_open;
    }

    fn render_into_texture(
        tiles: &Vec<Tile>, texture: &mut TextureHandle, boundary: usize, width: usize,
        height: usize,
    ) {
        let mut pixels = vec![Color32::BLACK; width * height];

        for (idx, tile) in tiles.iter().enumerate() {
            for y in 0..8 {
                for x in 0..8 {
                    // 16 tiles per row
                    let color: Color = tile.pixels[y][x].into();
                    let color32 =
                        Color32::from_rgba_premultiplied(color[0], color[1], color[2], 255);

                    let tile_x = (idx % boundary) * 8 + x;
                    let tile_y = (idx / boundary) * 8 + y;

                    pixels[tile_y * 8 * boundary + tile_x] = color32;
                }
            }
        }

        let image = ColorImage {
            size: [width, height],
            pixels,
        };

        texture.set(image, TextureOptions::NEAREST);
    }
}
