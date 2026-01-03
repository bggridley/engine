use anyhow::Result;
use std::sync::Arc;
use ash::vk;
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
    descriptor_set: vk::DescriptorSet,
    sampler: vk::Sampler,
    device: Arc<ash::Device>,
}

impl TextComponent {
    /// Helper function to build text vertices
    fn build_text_vertices(text: &str, font_atlas: &FontAtlas, font_size: f32) -> Vec<TexturedVertex2D> {
        let mut vertices = Vec::new();
        let scale = 0.5;  // Atlas is at 2x font_size
        
        let total_width: f32 = text.chars().filter_map(|ch| {
            font_atlas.get_glyph(ch).map(|g| g.advance_width * scale)
        }).sum();

        let start_x = -total_width / 2.0;
        let baseline_y = font_size * 0.25;
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
    pub fn new(text: &str, font_atlas: Arc<FontAtlas>, font_size: f32, context: &Arc<crate::renderer::VulkanContext>) -> Result<Self> {
        let device = context.device.clone();
        let vertices = Self::build_text_vertices(text, &font_atlas, font_size);

        let vertex_buffer = VertexBuffer::new(&context.device, context.physical_device, &context.instance, &vertices)?;

        // Create sampler for font texture
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);
        
        let sampler = unsafe { device.create_sampler(&sampler_info, None)? };

        // Create descriptor pool
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLER,
                descriptor_count: 1,
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        // Get descriptor set layout from pipeline manager
        let descriptor_set_layout = {
            let bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ];
            
            let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&bindings);
            
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? }
        };

        // Allocate descriptor set
        let layouts = [descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info)?[0] };

        // Write descriptor set
        let image_info = [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(font_atlas.texture_view)];

        let sampler_info_write = [vk::DescriptorImageInfo::default()
            .sampler(sampler)];

        let descriptor_writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .image_info(&image_info),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .image_info(&sampler_info_write),
        ];

        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };

        Ok(TextComponent {
            text: text.to_string(),
            font_atlas,
            transform: Transform2D::new(),
            color: [1.0, 1.0, 1.0],
            font_size,
            mesh: Mesh::new(vertex_buffer),
            descriptor_set,
            sampler,
            device: Arc::clone(&*context.device),
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
        
        self.text = text.to_string();
        let vertices = Self::build_text_vertices(text, &self.font_atlas, self.font_size);
        let vertex_buffer = VertexBuffer::new(&self.device, context.physical_device, &context.instance, &vertices)?;
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
            &[self.descriptor_set],
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
