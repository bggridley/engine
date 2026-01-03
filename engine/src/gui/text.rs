use anyhow::Result;
use std::sync::Arc;
use crate::gui::{GUIComponent, Transform2D};
use crate::renderer::{RenderContext, Renderer, FontAtlas, TexturedVertex2D, VertexBuffer, Mesh, PipelineId, PushConstants2D};
use glam::Vec2;

/// A text rendering component that displays text using a font atlas
pub struct TextComponent {
    text: String,
    font_atlas: Arc<FontAtlas>,
    transform: Transform2D,
    color: [f32; 3],
    font_size: f32,
    mesh: Mesh<TexturedVertex2D>,
}

impl TextComponent {
    /// Create a new text component
    pub fn new(text: &str, font_atlas: Arc<FontAtlas>, font_size: f32, context: &Arc<crate::renderer::VulkanContext>) -> Result<Self> {
        // Build vertices for the text
        let mut vertices = Vec::new();
        let scale = font_size / 128.0;
        let mut x = 0.0f32;
        
        // First pass: calculate total width for centering
        let total_width: f32 = text.chars().map(|ch| {
            if let Some(glyph) = font_atlas.get_glyph(ch) {
                glyph.width * scale
            } else {
                0.0
            }
        }).sum();

        let start_x = -total_width / 2.0;  // Center horizontally
        let start_y = -font_size / 2.0;     // Center vertically (approximate)
        let mut x = start_x;

        for ch in text.chars() {
            if let Some(glyph) = font_atlas.get_glyph(ch) {
                let width = glyph.width * scale;
                let height = glyph.height * scale;

                // First triangle
                vertices.push(TexturedVertex2D {
                    position: [x, start_y],
                    uv: [glyph.uv_min.x, glyph.uv_min.y],
                });
                vertices.push(TexturedVertex2D {
                    position: [x + width, start_y],
                    uv: [glyph.uv_max.x, glyph.uv_min.y],
                });
                vertices.push(TexturedVertex2D {
                    position: [x, start_y + height],
                    uv: [glyph.uv_min.x, glyph.uv_max.y],
                });

                // Second triangle
                vertices.push(TexturedVertex2D {
                    position: [x + width, start_y],
                    uv: [glyph.uv_max.x, glyph.uv_min.y],
                });
                vertices.push(TexturedVertex2D {
                    position: [x, start_y + height],
                    uv: [glyph.uv_min.x, glyph.uv_max.y],
                });

                x += width;
            }
        }

        let vertex_buffer = VertexBuffer::new(&context.device, context.physical_device, &context.instance, &vertices)?;

        Ok(TextComponent {
            text: text.to_string(),
            font_atlas,
            transform: Transform2D::new(),
            color: [1.0, 1.0, 1.0],
            font_size,
            mesh: Mesh::new(vertex_buffer),
        })
    }

    /// Set the text color (RGB)
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }

    /// Update the text content
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        // TODO: Rebuild mesh when text changes
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
}
