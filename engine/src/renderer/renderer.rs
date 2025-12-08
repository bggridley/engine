use crate::renderer::{CommandPool, FrameSynchronizer, Swapchain, VulkanContext};
use anyhow::Result;
use ash::{vk, Device};
use std::sync::Arc;

/// High-level rendering context for command recording
pub struct RenderContext {
    device: Arc<Device>,
    cmd_buffer: vk::CommandBuffer,
    extent: vk::Extent2D,
}

impl RenderContext {
    fn new(device: Arc<Device>, cmd_buffer: vk::CommandBuffer, extent: vk::Extent2D) -> Self {
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
    fn set_full_viewport(&self) {
        let viewport = vk::Viewport::default()
            .width(self.extent.width as f32)
            .height(self.extent.height as f32)
            .max_depth(1.0);
        unsafe {
            self.device.cmd_set_viewport(self.cmd_buffer, 0, &[viewport]);
        }
    }

    /// Set scissor to full extent
    fn set_full_scissor(&self) {
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
            self.device.cmd_draw(
                self.cmd_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }
}

/// Trait for anything that can be rendered
pub trait Renderable {
    fn render(&self, ctx: &RenderContext) -> Result<()>;
}

pub struct Renderer {
    context: Arc<VulkanContext>,
    swapchain: Swapchain,
    swapchain_loader: Arc<ash::khr::swapchain::Device>,
    command_pool: CommandPool,
    frame_sync: FrameSynchronizer,
    graphics_queue: vk::Queue,
    needs_rebuild: bool,
}

impl Renderer {
    pub fn new(context: Arc<VulkanContext>, width: u32, height: u32) -> Result<Self> {
        let swapchain_loader = Arc::new(ash::khr::swapchain::Device::new(&context.instance, &context.device));
        
        let swapchain = Swapchain::new(
            &context.device,
            &swapchain_loader,
            vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_SRGB,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            },
            vk::Extent2D { width, height },
            context.surface,
            vk::PresentModeKHR::FIFO,
            2,
            &context.queue_family_indices,
        );

        let command_pool = CommandPool::new(&context.device, context.queue_family_indices[0], 2);
        let frame_sync = FrameSynchronizer::new(&context.device, 2);
        
        let graphics_queue = unsafe {
            context.device.get_device_queue(context.queue_family_indices[0], 0)
        };

        Ok(Self {
            context,
            swapchain,
            swapchain_loader,
            command_pool,
            frame_sync,
            graphics_queue,
            needs_rebuild: false,
        })
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.swapchain.recreate(vk::Extent2D { width, height });
        }
    }

    pub fn begin_frame(&mut self) -> Option<RenderFrame<'_>> {
        // Handle swapchain rebuild if needed
        if self.needs_rebuild {
            self.needs_rebuild = false;
            return None;
        }

        self.frame_sync.begin_frame().ok()?;

        let image_index = match unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                self.frame_sync.current_image_available_semaphore(),
                vk::Fence::null(),
            )
        } {
            Ok((idx, _)) => idx,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.needs_rebuild = true;
                return None;
            }
            Err(_) => {
                self.frame_sync.end_frame();
                return None;
            }
        };

        let cmd_buffer = self.command_pool.buffers[self.frame_sync.current_frame_index()];

        // Reset and begin command buffer
        unsafe {
            self.context.device.reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).ok()?;
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.context.device.begin_command_buffer(cmd_buffer, &begin_info).ok()?;
        }

        let render_ctx = RenderContext::new(
            self.context.device.clone(),
            cmd_buffer,
            self.swapchain.extent,
        );

        // Transition to render target
        render_ctx.transition_image(
            self.swapchain.images[image_index as usize],
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

        // Begin rendering
        render_ctx.begin_rendering(
            self.swapchain.image_views[image_index as usize],
            [0.25, 0.1, 0.1, 1.0],
        );

        Some(RenderFrame {
            renderer: self,
            render_ctx,
            image_index,
            cmd_buffer,
        })
    }

    fn end_frame(&mut self, image_index: u32, cmd_buffer: vk::CommandBuffer) {
        unsafe {
            self.context.device.end_command_buffer(cmd_buffer).ok();
        }

        // Submit and present
        unsafe {
            let cmd_buffers = [cmd_buffer];
            let wait_sems = [self.frame_sync.current_image_available_semaphore()];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_sems = [self.frame_sync.current_render_finished_semaphore()];

            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&cmd_buffers)
                .wait_semaphores(&wait_sems)
                .wait_dst_stage_mask(&wait_stages)
                .signal_semaphores(&signal_sems);

            self.context.device.queue_submit(self.graphics_queue, &[submit_info], self.frame_sync.current_in_flight_fence()).ok();

            let signal_sems_present = [self.frame_sync.current_render_finished_semaphore()];
            let swapchains = [self.swapchain.swapchain];
            let image_indices = [image_index];

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_sems_present)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            let _ = self.swapchain_loader.queue_present(self.graphics_queue, &present_info);
        }

        self.frame_sync.end_frame();
    }
}

pub struct RenderFrame<'a> {
    renderer: &'a mut Renderer,
    render_ctx: RenderContext,
    image_index: u32,
    cmd_buffer: vk::CommandBuffer,
}

impl<'a> RenderFrame<'a> {
    pub fn render_context(&self) -> &RenderContext {
        &self.render_ctx
    }
}

impl<'a> Drop for RenderFrame<'a> {
    fn drop(&mut self) {
        // End rendering
        self.render_ctx.end_rendering();

        // Transition to present
        self.render_ctx.transition_image(
            self.renderer.swapchain.images[self.image_index as usize],
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        );

        self.renderer.end_frame(self.image_index, self.cmd_buffer);
    }
}
