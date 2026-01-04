use anyhow::Result;
use std::sync::Arc;
use ash::vk;
use crate::gui::{GUIComponent, Transform2D};
use crate::renderer::{RenderContext, Renderer, FontAtlas, TexturedVertex2D, VertexBuffer, Mesh, PipelineId, PushConstants2D, SampledTexture, SamplerConfig};
use glam::Vec2;

/// A text rendering component that displays text using a font atlas
pub struct TextComponent {
    text: String,
    font_atlas: Arc<FontAtlas>,
    transform: Transform2D,
    color: [f32; 3],
    font_size: f32,
    mesh: Mesh<TexturedVertex2D>,
    sampled_texture: SampledTexture,
}

impl TextComponent {
    /// Helper function to build text vertices
    fn build_text_vertices(text: &str, font_atlas: &FontAtlas, font_size: f32) -> Vec<TexturedVertex2D> {
        let mut vertices = Vec::new();
        let scale = 0.5;  // Atlas is at 2x font_size
        
        let total_width: f32 = text.chars().filter_map(|ch| {
            font_atlas.get_glyph(ch).map(|g| g.advance_width * scale)
        }).sum();

        // Calculate the actual height bounds of the text to center it vertically
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for ch in text.chars() {
            if let Some(glyph) = font_atlas.get_glyph(ch) {
                let bearing_y = glyph.bearing_y * scale;
                let height = glyph.height * scale;
                let top = -bearing_y;
                let bottom = top + height;
                min_y = min_y.min(top);
                max_y = max_y.max(bottom);
            }
        }
        
        // Center the text vertically around y=0
        let text_height = max_y - min_y;
        let baseline_y = -text_height / 2.0 - min_y;
        
        let start_x = -total_width / 2.0;
        let mut x = start_x;

        for ch in text.chars() {
            if let Some(glyph) = font_atlas.get_glyph(ch) {
                let width = glyph.width * scale;
                let height = glyph.height * scale;
                
                if width > 0.0 && height > 0.0 {
                    let bearing_y = glyph.bearing_y * scale;
                    let y = baseline_y - bearing_y;

                    vertices.push(TexturedVertex2D {
                        position: [x, y],
                        uv: [glyph.uv_min.x, glyph.uv_min.y],
                    });
                    vertices.push(TexturedVertex2D {
                        position: [x + width, y],
                        uv: [glyph.uv_max.x, glyph.uv_min.y],
                    });
                    vertices.push(TexturedVertex2D {
                        position: [x, y + height],
                        uv: [glyph.uv_min.x, glyph.uv_max.y],
                    });
                    vertices.push(TexturedVertex2D {
                        position: [x + width, y],
                        uv: [glyph.uv_max.x, glyph.uv_min.y],
                    });
                    vertices.push(TexturedVertex2D {
                        position: [x + width, y + height],
                        uv: [glyph.uv_max.x, glyph.uv_max.y],
                    });
                    vertices.push(TexturedVertex2D {
                        position: [x, y + height],
                        uv: [glyph.uv_min.x, glyph.uv_max.y],
                    });
                }

                x += glyph.advance_width * scale;
            }
        }
        
        vertices
    }
    
    /// Create a new text component
    /// 
    /// Requires the descriptor_set_layout from PipelineManager to avoid redundant layout creation.
    /// Get it via: renderer.get_descriptor_set_layout(PipelineId::Text).unwrap()
    pub fn new(
        text: &str,
        font_atlas: Arc<FontAtlas>,
        font_size: f32,
        descriptor_set_layout: vk::DescriptorSetLayout,
        context: &Arc<crate::renderer::VulkanContext>,
    ) -> Result<Self> {
        let vertices = Self::build_text_vertices(text, &font_atlas, font_size);

        let vertex_buffer = VertexBuffer::new(&context.device, context.physical_device, &context.instance, &vertices)?;

        // Create sampled texture with linear filtering for smooth text
        let sampled_texture = SampledTexture::new(
            &font_atlas.texture,
            SamplerConfig::linear(),
            descriptor_set_layout,
            &context.device,
        )?;

        Ok(TextComponent {
            text: text.to_string(),
            font_atlas,
            transform: Transform2D::new(),
            color: [1.0, 1.0, 1.0],
            font_size,
            mesh: Mesh::new(vertex_buffer),
            sampled_texture,
        })
    }

    /// Set the text color (RGB)
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }

    /// Update the text content and rebuild mesh
    pub fn update_text(&mut self, text: &str, context: &Arc<crate::renderer::VulkanContext>) -> Result<()> {
        // Only rebuild if text actually changed
        if self.text == text {
            return Ok(());
        }
        
        // Wait for GPU to finish using old mesh before destroying it
        unsafe {
            let _ = context.device.device_wait_idle();
        }
        
        // Destroy old mesh before creating new one
        self.mesh.destroy(&context.device);
        
        self.text = text.to_string();
        let vertices = Self::build_text_vertices(text, &self.font_atlas, self.font_size);
        let vertex_buffer = VertexBuffer::new(&context.device, context.physical_device, &context.instance, &vertices)?;
        self.mesh = Mesh::new(vertex_buffer);
        Ok(())
    }

    /// Get the width of the current text at the given font size
    pub fn get_width(&self) -> f32 {
        self.font_atlas.get_text_width(&self.text) * (self.font_size / 128.0)
    }

    /// Get the height (approximate, based on font size)
    pub fn get_height(&self) -> f32 {
        self.font_size
    }

    /// Set the text position
    pub fn set_position(&mut self, position: Vec2) {
        self.transform.position = position;
    }
}

impl GUIComponent for TextComponent {
    fn render(&self, ctx: &RenderContext, renderer: &mut Renderer) -> Result<()> {
        let pipeline = renderer.get_pipeline(PipelineId::Text)?;
        let pipeline_layout = renderer.get_pipeline_layout(PipelineId::Text)
            .ok_or_else(|| anyhow::anyhow!("Pipeline layout not found for Text pipeline"))?;
        ctx.bind_pipeline(pipeline);

        // Bind descriptor set for font texture
        ctx.bind_descriptor_sets(
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[self.sampled_texture.descriptor_set],
            &[],
        );

        let push = PushConstants2D {
            projection: renderer.projection,
            transform: glam::Mat4::from_translation(glam::Vec3::new(
                self.transform.position.x,
                self.transform.position.y,
                0.0,
            )) * glam::Mat4::from_scale(glam::Vec3::new(
                self.transform.scale.x,
                self.transform.scale.y,
                1.0,
            )),
            color_modulation: self.color,  // Use text color
            _padding: 0.0,
        };

        ctx.push_constants(pipeline_layout, &push);
        self.mesh.draw(ctx)?;

        Ok(())
    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    fn handle_mouse_down(&mut self, _x: f32, _y: f32) {
        // Text doesn't handle input yet
    }

    fn handle_mouse_up(&mut self, _x: f32, _y: f32) {
        // Text doesn't handle input yet
    }

    fn handle_mouse_move(&mut self, _x: f32, _y: f32) {
        // Text doesn't handle input yet
    }

    fn destroy(&self, device: &ash::Device) {
        self.mesh.destroy(device);
        self.sampled_texture.destroy(device);
    }
}

