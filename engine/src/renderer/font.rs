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
    pub bearing_x: f32,  // Horizontal bearing (offset from cursor)
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

        // Rasterize each character individually (not as a laid-out string)
        let glyphs: Vec<_> = CHARS_TO_RASTERIZE
            .chars()
            .filter_map(|ch| {
                let glyph = font.glyph(ch).scaled(scale);
                let h_metrics = glyph.h_metrics();
                let positioned = glyph.positioned(point(0.0, v_metrics.ascent));
                positioned.pixel_bounding_box().map(|bb| (ch, positioned, bb, h_metrics))
            })
            .collect();

        // Calculate texture dimensions - pack glyphs horizontally
        let mut texture_width = 0usize;
        let texture_height = height.ceil() as usize;
        
        for (_, _, bb, _) in &glyphs {
            texture_width += (bb.max.x - bb.min.x) as usize + 2; // Add padding
        }
        texture_width = texture_width.max(512);

        // Create pixel buffer
        let mut pixels = vec![0u8; texture_width * texture_height];

        // Draw glyphs and build metrics map
        let mut x_offset = 0i32;
        let mut glyph_map = HashMap::new();
        
        for (ch, positioned_glyph, bb, h_metrics) in glyphs {
            // Draw glyph at x_offset
            positioned_glyph.draw(|x, y, v| {
                let px = (x_offset + x as i32) as usize;
                let py = (texture_height as i32 - 1 - (bb.min.y + y as i32)) as usize;
                if px < texture_width && py < texture_height {
                    pixels[py * texture_width + px] = (v * 255.0) as u8;
                }
            });

            let width = (bb.max.x - bb.min.x) as f32;
            let height = (bb.max.y - bb.min.y) as f32;
            
            glyph_map.insert(
                ch,
                GlyphMetrics {
                    uv_min: Vec2::new(
                        x_offset as f32 / texture_width as f32,
                        (texture_height as i32 - bb.max.y) as f32 / texture_height as f32,
                    ),
                    uv_max: Vec2::new(
                        (x_offset + width as i32) as f32 / texture_width as f32,
                        (texture_height as i32 - bb.min.y) as f32 / texture_height as f32,
                    ),
                    advance_width: h_metrics.advance_width,
                    bearing_x: bb.min.x as f32,
                    bearing_y: bb.max.y as f32,
                    width,
                    height,
                },
            );
            
            x_offset += width as i32 + 2; // Add padding between glyphs
        }
        
        // Add space character manually
        if let Some(glyph) = font.glyph(' ').scaled(scale).h_metrics().advance_width.into() {
            let space_advance: f32 = glyph;
            glyph_map.insert(
                ' ',
                GlyphMetrics {
                    uv_min: Vec2::ZERO,
                    uv_max: Vec2::ZERO,
                    advance_width: space_advance,
                    bearing_x: 0.0,
                    bearing_y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
            );
        }

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