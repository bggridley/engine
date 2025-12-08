use anyhow::Result;
use engine::window::EventLoop;
use engine::renderer::{VulkanContext, Swapchain, CommandPool, FrameSynchronizer, DynamicRenderingAttachment, TriangleRenderer};
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    window::WindowBuilder,
};
use ash::khr::swapchain::Device as SwapchainDevice;
use ash::vk;

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    
    // Create window
    let window = WindowBuilder::new()
        .with_title("Vulkan Engine")
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop)?;
    let window = Arc::new(window);

    // Initialize Vulkan
    let context = Arc::new(VulkanContext::new(window.clone())?);
    let swapchain_loader = SwapchainDevice::new(&context.instance, &context.device);
    
    // Create swapchain
    let swapchain = Swapchain::new(
        &context.device,
        &swapchain_loader,
        ash::vk::SurfaceFormatKHR {
            format: ash::vk::Format::B8G8R8A8_SRGB,
            color_space: ash::vk::ColorSpaceKHR::SRGB_NONLINEAR,
        },
        ash::vk::Extent2D {
            width: 1280,
            height: 720,
        },
        context.surface,
        ash::vk::PresentModeKHR::FIFO,
        2,
        &context.queue_family_indices,
    );
    
    // Create command pool and synchronization
    let command_pool = CommandPool::new(
        &context.device,
        context.queue_family_indices[0],
        2,
    );
    let mut frame_sync = FrameSynchronizer::new(&context.device, 2);
    
    // Get graphics queue
    let graphics_queue = unsafe {
        context.device.get_device_queue(context.queue_family_indices[0], 0)
    };
    
    // Create triangle renderer
    let triangle_renderer = TriangleRenderer::new(&context.device, &context.instance, context.physical_device)?;

    println!("Vulkan Engine initialized!");
    println!("Window: {:?}", window.id());
    println!("Swapchain images: {}", swapchain.images.len());

    let mut frame_count = 0u32;

    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent { event: window_event, .. } => match window_event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                WindowEvent::RedrawRequested => {
                    // Wait for previous frame and begin new frame
                    frame_sync.begin_frame().ok();
                    
                    // Get frame synchronization primitives
                    let frame_index = frame_sync.current_frame_index();
                    let image_available = frame_sync.current_image_available_semaphore();
                    let render_finished = frame_sync.current_render_finished_semaphore();
                    let in_flight_fence = frame_sync.current_in_flight_fence();
                    
                    // Acquire swapchain image
                    let (image_index, _) = match unsafe {
                        swapchain_loader.acquire_next_image(
                            swapchain.swapchain,
                            u64::MAX,
                            image_available,
                            vk::Fence::null(),
                        )
                    } {
                        Ok(result) => result,
                        Err(_) => return,
                    };
                    
                    // Get command buffer
                    let cmd_buffer = command_pool.buffers[frame_index];
                    
                    // Record command buffer
                    unsafe {
                        context.device.reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).ok();
                        
                        let begin_info = vk::CommandBufferBeginInfo::default()
                            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
                        context.device.begin_command_buffer(cmd_buffer, &begin_info).ok();
                    }
                    
                    // Transition image
                    unsafe {
                        let barrier = vk::ImageMemoryBarrier::default()
                            .old_layout(vk::ImageLayout::UNDEFINED)
                            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                            .src_access_mask(vk::AccessFlags::empty())
                            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                            .image(swapchain.images[image_index as usize])
                            .subresource_range(
                                vk::ImageSubresourceRange::default()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .level_count(1)
                                    .layer_count(1),
                            );
                        
                        context.device.cmd_pipeline_barrier(
                            cmd_buffer,
                            vk::PipelineStageFlags::TOP_OF_PIPE,
                            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            vk::DependencyFlags::empty(),
                            &[],
                            &[],
                            &[barrier],
                        );
                    }
                    
                    // Begin rendering and draw triangle
                    unsafe {
                        let color_attachment = DynamicRenderingAttachment::color(
                            swapchain.image_views[image_index as usize],
                            vk::AttachmentLoadOp::CLEAR,
                            vk::AttachmentStoreOp::STORE,
                        );
                        
                        let color_attachment_info = vk::RenderingAttachmentInfo::default()
                            .image_view(color_attachment.image_view)
                            .image_layout(color_attachment.image_layout)
                            .load_op(color_attachment.load_op)
                            .store_op(color_attachment.store_op)
                            .clear_value(color_attachment.clear_value);
                        
                        let rendering_info = vk::RenderingInfo::default()
                            .render_area(vk::Rect2D::default().extent(swapchain.extent))
                            .layer_count(1)
                            .color_attachments(std::slice::from_ref(&color_attachment_info));
                        
                        context.device.cmd_begin_rendering(cmd_buffer, &rendering_info);
                        
                        // Set viewport and scissor
                        let viewport = vk::Viewport::default()
                            .width(swapchain.extent.width as f32)
                            .height(swapchain.extent.height as f32)
                            .max_depth(1.0);
                        context.device.cmd_set_viewport(cmd_buffer, 0, &[viewport]);
                        
                        let scissor = vk::Rect2D::default().extent(swapchain.extent);
                        context.device.cmd_set_scissor(cmd_buffer, 0, &[scissor]);
                        
                        // Draw triangle
                        triangle_renderer.draw(&context.device, cmd_buffer);
                        
                        context.device.cmd_end_rendering(cmd_buffer);
                    }
                    
                    // Transition image back
                    unsafe {
                        let barrier = vk::ImageMemoryBarrier::default()
                            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                            .dst_access_mask(vk::AccessFlags::empty())
                            .image(swapchain.images[image_index as usize])
                            .subresource_range(
                                vk::ImageSubresourceRange::default()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .level_count(1)
                                    .layer_count(1),
                            );
                        
                        context.device.cmd_pipeline_barrier(
                            cmd_buffer,
                            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                            vk::DependencyFlags::empty(),
                            &[],
                            &[],
                            &[barrier],
                        );
                        
                        context.device.end_command_buffer(cmd_buffer).ok();
                    }
                    
                    // Submit and present
                    unsafe {
                        let cmd_buffers = [cmd_buffer];
                        let wait_semaphores = [image_available];
                        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                        let signal_semaphores = [render_finished];
                        
                        let submit_info = vk::SubmitInfo::default()
                            .command_buffers(&cmd_buffers)
                            .wait_semaphores(&wait_semaphores)
                            .wait_dst_stage_mask(&wait_stages)
                            .signal_semaphores(&signal_semaphores);
                        
                        context.device.queue_submit(graphics_queue, &[submit_info], in_flight_fence).ok();
                        
                        let render_finished_sems = [render_finished];
                        let swapchains = [swapchain.swapchain];
                        let image_indices = [image_index];
                        let present_info = vk::PresentInfoKHR::default()
                            .wait_semaphores(&render_finished_sems)
                            .swapchains(&swapchains)
                            .image_indices(&image_indices);
                        
                        let _ = swapchain_loader.queue_present(graphics_queue, &present_info);
                    }
                    
                    frame_sync.end_frame();
                    frame_count += 1;
                    
                    if frame_count % 60 == 0 {
                        println!("Frames: {}", frame_count);
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}