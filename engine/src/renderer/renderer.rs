use crate::renderer::{CommandPool, FrameSynchronizer, PipelineManager, Swapchain, VulkanContext};
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

    /// Bind pipeline directly
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

    /// Bind index buffer
    pub fn bind_index_buffer(&self, buffer: vk::Buffer) {
        unsafe {
            self.device.cmd_bind_index_buffer(self.cmd_buffer, buffer, 0, vk::IndexType::UINT32);
        }
    }

    /// Push constants (fast per-draw uniforms)
    pub fn push_constants<T>(&self, layout: vk::PipelineLayout, data: &T) {
        unsafe {
            let bytes = std::slice::from_raw_parts(
                data as *const T as *const u8,
                std::mem::size_of::<T>(),
            );
            self.device.cmd_push_constants(
                self.cmd_buffer,
                layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytes,
            );
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

    /// Draw indexed vertices
    pub fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        unsafe {
            self.device.cmd_draw_indexed(
                self.cmd_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }
}


pub struct Renderer {
    context: Arc<VulkanContext>,
    swapchain: Swapchain,
    swapchain_loader: Arc<ash::khr::swapchain::Device>,
    command_pool: CommandPool,
    frame_sync: FrameSynchronizer,
    pipeline_manager: PipelineManager,
    graphics_queue: vk::Queue,
    needs_rebuild: bool,
    current_frame: usize,
    pub projection: glam::Mat4,
}

impl Renderer {
    pub fn new(context: Arc<VulkanContext>, width: u32, height: u32) -> Result<Self> {
        let swapchain_loader = Arc::new(ash::khr::swapchain::Device::new(&context.instance, &context.device));
        
        // Query supported surface formats
        let surface_formats = unsafe {
            context.surface_loader.get_physical_device_surface_formats(
                context.physical_device,
                context.surface,
            )?
        };

        // Find SRGB format or fall back to first available
        let surface_format = surface_formats
            .iter()
            .find(|fmt| fmt.format == vk::Format::B8G8R8A8_SRGB)
            .or_else(|| surface_formats.first())
            .copied()
            .unwrap_or(vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_SRGB,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            });

        println!("Selected surface format: {:?}", surface_format);
        
        let swapchain = Swapchain::new(
            &context.device,
            &swapchain_loader,
            surface_format,
            vk::Extent2D { width, height },
            context.surface,
            vk::PresentModeKHR::FIFO,
            2,
            &context.queue_family_indices,
        );

        // Use frames-in-flight pattern (2 = double buffering)
        let max_frames_in_flight = 2;
        let swapchain_image_count = swapchain.images.len();
        let command_pool = CommandPool::new(&context.device, context.queue_family_indices[0], max_frames_in_flight as u32);
        let frame_sync = FrameSynchronizer::new(&context.device, max_frames_in_flight, swapchain_image_count);
        
        let graphics_queue = unsafe {
            context.device.get_device_queue(context.queue_family_indices[0], 0)
        };

        // Compile shaders and build all pipelines up front
        let shader_manager = crate::renderer::ShaderManager::new()?;
        shader_manager.compile_all_shaders()?;
        
        let mut pipeline_manager = PipelineManager::new((*context.device).clone());
        pipeline_manager.build_all()?;

        Ok(Self {
            context,
            swapchain,
            swapchain_loader,
            command_pool,
            frame_sync,
            pipeline_manager,
            graphics_queue,
            needs_rebuild: false,
            current_frame: 0,
            projection: glam::Mat4::IDENTITY,
        })
    }

    pub fn handle_resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        // Only recreate if size actually changed
        if width > 0 && height > 0 && (width != self.swapchain.extent.width || height != self.swapchain.extent.height) {
            println!("Resizing swapchain: {}x{} -> {}x{}", self.swapchain.extent.width, self.swapchain.extent.height, width, height);
            self.swapchain.recreate(vk::Extent2D { width, height });
        }


        self.projection = glam::Mat4::orthographic_rh(
            0.0,
            width as f32,
            0.0,
            height as f32,
            -1.0,
            1.0,
        );

        println!("Updated projection matrix for new size: {:?}", self.projection);
    }

    pub fn begin_frame(&mut self) -> Option<RenderFrame> {
        // Handle swapchain rebuild if needed
        if self.needs_rebuild {
            self.needs_rebuild = false;
            return None;
        }

        // Wait for this frame's fence to be signaled (CPU-GPU sync)
        self.frame_sync.wait_for_frame(self.current_frame).ok()?;

        // Get acquire semaphore for this frame
        let image_available_sem = self.frame_sync.get_acquire_semaphore(self.current_frame);
        
        // Acquire next image
        let image_index = match unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                image_available_sem,
                vk::Fence::null(),
            )
        } {
            Ok((idx, _)) => idx,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.needs_rebuild = true;
                return None;
            }
            Err(_) => {
                return None;
            }
        };

        // Get the render finished semaphore for THIS SPECIFIC IMAGE
        let render_finished_sem = self.frame_sync.get_render_finished_semaphore(image_index);

        // Check if this image is still being used by a previous frame
        if let Some(image_fence) = self.frame_sync.images_in_flight[image_index as usize] {
            unsafe {
                self.context.device.wait_for_fences(&[image_fence], true, u64::MAX).ok()?;
            }
        }
        
        // Mark this image as in use by this frame
        self.frame_sync.images_in_flight[image_index as usize] = Some(self.frame_sync.get_fence(self.current_frame));

        // Reset fence for this frame
        self.frame_sync.reset_fence(self.current_frame).ok()?;

        let cmd_buffer = self.command_pool.buffers[self.current_frame];

        // Reset and begin command buffer
        unsafe {
            self.context.device.reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).ok()?;
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.context.device.begin_command_buffer(cmd_buffer, &begin_info).ok()?;
        }

        let render_ctx = RenderContext::new(
            Arc::clone(&self.context.device),
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

        let frame = RenderFrame {
            render_ctx,
            swapchain: self.swapchain.swapchain,
            swapchain_image: self.swapchain.images[image_index as usize],
            swapchain_loader: self.swapchain_loader.clone(),
            graphics_queue: self.graphics_queue,
            device: Arc::clone(&self.context.device),
            image_index,
            cmd_buffer,
            wait_semaphore: image_available_sem,
            signal_semaphore: render_finished_sem,
            fence: self.frame_sync.get_fence(self.current_frame),
        };

        // Advance to next frame (modulo max_frames_in_flight, not swapchain image count)
        self.current_frame = (self.current_frame + 1) % self.frame_sync.max_frames_in_flight();

        Some(frame)
    }

    /// Get a pipeline by ID
    pub fn get_pipeline(&mut self, id: crate::renderer::PipelineId) -> Result<vk::Pipeline> {
        self.pipeline_manager.get(id)
    }

    pub fn get_pipeline_layout(&self, id: crate::renderer::PipelineId) -> Option<vk::PipelineLayout> {
        self.pipeline_manager.get_layout(id)
    }
}

pub struct RenderFrame {
    pub render_ctx: RenderContext,
    swapchain: vk::SwapchainKHR,
    swapchain_image: vk::Image,
    swapchain_loader: Arc<ash::khr::swapchain::Device>,
    graphics_queue: vk::Queue,
    device: Arc<Device>,
    image_index: u32,
    cmd_buffer: vk::CommandBuffer,
    wait_semaphore: vk::Semaphore,
    signal_semaphore: vk::Semaphore,
    fence: vk::Fence,
}

impl RenderFrame {
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            // Wait for all GPU work to complete
            let _ = self.context.device.device_wait_idle();
            
            // Fields will be dropped in reverse order of declaration:
            // 1. current_frame (usize - no cleanup)
            // 2. needs_rebuild (bool - no cleanup)
            // 3. graphics_queue (vk::Queue - no cleanup needed, owned by device)
            // 4. pipeline_manager (has Drop impl - destroys pipelines)
            // 5. frame_sync (has Drop impl - destroys semaphores and fences)
            // 6. command_pool (has Drop impl - destroys pool)
            // 7. swapchain_loader (Arc - no cleanup)
            // 8. swapchain (has Drop impl - destroys swapchain and image views)
            // 9. context (Arc - may trigger VulkanContext::drop if last reference)
        }
    }
}

impl Drop for RenderFrame {
    fn drop(&mut self) {
        // End rendering
        self.render_ctx.end_rendering();

        // Transition to present
        self.render_ctx.transition_image(
            self.swapchain_image,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        );

        unsafe {
            self.device.end_command_buffer(self.cmd_buffer).ok();

            // Submit with fence for GPU-CPU synchronization
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(std::slice::from_ref(&self.wait_semaphore))
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(std::slice::from_ref(&self.cmd_buffer))
                .signal_semaphores(std::slice::from_ref(&self.signal_semaphore));

            self.device.queue_submit(self.graphics_queue, &[submit_info], self.fence).ok();

            // Present
            let swapchains = [self.swapchain];
            let image_indices = [self.image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(std::slice::from_ref(&self.signal_semaphore))
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            let _ = self.swapchain_loader.queue_present(self.graphics_queue, &present_info);
        }
    }
}
