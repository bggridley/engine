// Simplified engine/src/renderer/font.rs
use anyhow::Result;
use ash::vk::Format;
use glam::Vec2;
use rusttype::{point, Font, Scale};
use std::{collections::HashMap, sync::Arc};

use super::Texture;

pub struct FontAtlas {
    pub texture: Texture,
    pub glyph_map: HashMap<char, GlyphMetrics>,
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphMetrics {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub advance_width: f32,
    pub bearing_y: f32,
    pub width: f32,   // Pixel width in the rasterized texture
    pub height: f32,  // Pixel height in the rasterized texture
}

const CHARS_TO_RASTERIZE: &str =
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

impl FontAtlas {
    pub fn load(
        path: &str,
        font_size: f32,
        device: &Arc<ash::Device>,
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self> {
        let font_data = std::fs::read(path)
            .map_err(|e| anyhow::anyhow!("Failed to load font file '{}': {}", path, e))?;
        let font = Font::try_from_vec(font_data)
            .ok_or_else(|| anyhow::anyhow!("Invalid font file format"))?;

        // Rasterize at 2x target size for good antialiasing, then scale down 2x for crisp rendering
        let height: f32 = font_size * 2.0;
        let scale = Scale { x: height, y: height };

        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        let glyphs: Vec<_> = font.layout(CHARS_TO_RASTERIZE, scale, offset).collect();

        // Calculate texture width from rightmost glyph
        let texture_width = glyphs
            .iter()
            .map(|g| {
                let bb = g.pixel_bounding_box().unwrap_or_default();
                (bb.max.x as u32)
                    .max(g.position().x as u32 + g.unpositioned().h_metrics().advance_width as u32)
            })
            .max()
            .unwrap_or(512) as usize;

        let texture_height = height.ceil() as usize;

        // Create pixel buffer
        let mut pixels = vec![0u8; texture_width * texture_height];

        // Draw glyphs into buffer
        for glyph in &glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let px = (bb.min.x + x as i32) as usize;
                    let py = (texture_height as i32 - 1 - (bb.min.y + y as i32)) as usize;
                    if px < texture_width && py < texture_height {
                        pixels[py * texture_width + px] = (v * 255.0) as u8;
                    }
                });
            }
        }

        // Build glyph map
        let glyph_map = glyphs
            .iter()
            .zip(CHARS_TO_RASTERIZE.chars())
            .map(|(g, ch)| {
                let advance_width = g.unpositioned().h_metrics().advance_width;
                
                // Handle glyphs without bounding boxes (e.g., space)
                if let Some(bb) = g.pixel_bounding_box() {
                    let width = (bb.max.x - bb.min.x) as f32;
                    let height = (bb.max.y - bb.min.y) as f32;
                    (
                        ch,
                        GlyphMetrics {
                            uv_min: Vec2::new(
                                bb.min.x as f32 / texture_width as f32,
                                (texture_height as i32 - bb.max.y) as f32 / texture_height as f32,
                            ),
                            uv_max: Vec2::new(
                                bb.max.x as f32 / texture_width as f32,
                                (texture_height as i32 - bb.min.y) as f32 / texture_height as f32,
                            ),
                            advance_width,
                            bearing_y: bb.max.y as f32,
                            width,
                            height,
                        },
                    )
                } else {
                    // For invisible characters like space, just store advance width
                    (
                        ch,
                        GlyphMetrics {
                            uv_min: Vec2::ZERO,
                            uv_max: Vec2::ZERO,
                            advance_width,
                            bearing_y: 0.0,
                            width: 0.0,
                            height: 0.0,
                        },
                    )
                }
            })
            .collect();

        // Use the reusable Texture module to create the GPU texture
        let texture = Texture::from_bytes(
            &pixels,
            texture_width as u32,
            texture_height as u32,
            Format::R8_UNORM,  // Single-channel grayscale for font
            device,
            instance,
            physical_device,
            queue_family_index,
        )?;

        Ok(FontAtlas { texture, glyph_map })
    }

    pub fn get_text_width(&self, text: &str) -> f32 {
        text.chars()
            .filter_map(|c| self.glyph_map.get(&c))
            .map(|metrics| metrics.advance_width)
            .sum()
    }

    pub fn get_glyph(&self, ch: char) -> Option<&GlyphMetrics> {
        self.glyph_map.get(&ch)
    }

    /// Manually destroy Vulkan resources
    pub fn destroy(&self, device: &ash::Device) {
        self.texture.destroy(device);
    }
}