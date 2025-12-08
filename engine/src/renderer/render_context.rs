use ash::{vk, Device};
use anyhow::Result;
use std::sync::Arc;

/// High-level rendering context that abstracts Vulkan command recording
pub struct RenderContext {
    pub device: Arc<Device>,
    pub cmd_buffer: vk::CommandBuffer,
    pub extent: vk::Extent2D,
}

impl RenderContext {
    pub fn new(
        device: Arc<Device>,
        cmd_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) -> Self {
        RenderContext {
            device,
            cmd_buffer,
            extent,
        }
    }

    /// Begin a rendering pass with a color attachment
    pub fn begin_rendering(&self, image_view: vk::ImageView, clear_color: [f32; 4]) {
        unsafe {
            let color_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    color: vk::ClearColorValue { float32: clear_color },
                });

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(self.extent))
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment));

            self.device.cmd_begin_rendering(self.cmd_buffer, &rendering_info);
        }

        // Set viewport and scissor
        self.set_full_viewport();
        self.set_full_scissor();
    }

    /// End the rendering pass
    pub fn end_rendering(&self) {
        unsafe {
            self.device.cmd_end_rendering(self.cmd_buffer);
        }
    }

    /// Transition image layout
    pub fn transition_image(
        &self,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
    ) {
        unsafe {
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(old_layout)
                .new_layout(new_layout)
                .src_access_mask(match old_layout {
                    vk::ImageLayout::UNDEFINED => vk::AccessFlags::empty(),
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    _ => vk::AccessFlags::empty(),
                })
                .dst_access_mask(match new_layout {
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    vk::ImageLayout::PRESENT_SRC_KHR => vk::AccessFlags::empty(),
                    _ => vk::AccessFlags::empty(),
                })
                .image(image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                );

            self.device.cmd_pipeline_barrier(
                self.cmd_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }
    }

    /// Set viewport to full extent
    pub fn set_full_viewport(&self) {
        let viewport = vk::Viewport::default()
            .width(self.extent.width as f32)
            .height(self.extent.height as f32)
            .max_depth(1.0);
        unsafe {
            self.device.cmd_set_viewport(self.cmd_buffer, 0, &[viewport]);
        }
    }

    /// Set scissor to full extent
    pub fn set_full_scissor(&self) {
        let scissor = vk::Rect2D::default().extent(self.extent);
        unsafe {
            self.device.cmd_set_scissor(self.cmd_buffer, 0, &[scissor]);
        }
    }

    /// Bind a pipeline
    pub fn bind_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe {
            self.device.cmd_bind_pipeline(
                self.cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
        }
    }

    /// Bind vertex buffer
    pub fn bind_vertex_buffer(&self, buffer: vk::Buffer) {
        unsafe {
            self.device.cmd_bind_vertex_buffers(self.cmd_buffer, 0, &[buffer], &[0]);
        }
    }

    /// Draw vertices
    pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            self.device
                .cmd_draw(self.cmd_buffer, vertex_count, instance_count, first_vertex, first_instance);
        }
    }
}

/// Trait for anything that can be rendered
pub trait Renderable {
    fn render(&self, ctx: &RenderContext) -> Result<()>;
}
