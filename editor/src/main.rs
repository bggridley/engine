use anyhow::Result;
use engine::window::EventLoop;
use engine::renderer::{VulkanContext, Swapchain, CommandPool, FrameSynchronizer, DynamicRenderingAttachment};
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    window::WindowBuilder,
};
use ash::khr::swapchain::Device as SwapchainDevice;
use ash::vk;
use std::time::Instant;

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    
    // Create window
    let window = WindowBuilder::new()
        .with_title("Vulkan Editor")
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop)?;
    let window = Arc::new(window);

    // Initialize Vulkan context from engine
    let context = Arc::new(VulkanContext::new(window.clone())?);
    
    // Initialize swapchain loader
    let swapchain_loader = SwapchainDevice::new(&context.instance, &context.device);
    
    // Initialize swapchain
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
    
    // Initialize command pool
    let command_pool = CommandPool::new(
        &context.device,
        context.queue_family_indices[0],
        2,
    );
    
    // Initialize synchronization
    let mut frame_sync = FrameSynchronizer::new(&context.device, 2);
    
    // Get graphics queue
    let graphics_queue = unsafe {
        context.device.get_device_queue(context.queue_family_indices[0], 0)
    };
    
    // Initialize ImGui
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        window.as_ref(),
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.style_mut().window_rounding = 0.0;
    
    // Create a dummy render pass for ImGui (required by renderer even though we use dynamic rendering)
    let render_pass = unsafe {
        let attachment = vk::AttachmentDescription::default()
            .format(vk::Format::B8G8R8A8_SRGB)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&attachment))
            .subpasses(std::slice::from_ref(&subpass));

        context.device.create_render_pass(&render_pass_info, None)
            .expect("Failed to create render pass")
    };
    
    // Initialize ImGui Vulkan renderer
    let mut renderer = imgui_rs_vulkan_renderer::Renderer::with_default_allocator(
        &context.instance,
        context.graphics_devices[0].0,
        context.device.as_ref().clone(),
        graphics_queue,
        command_pool.pool,
        render_pass,
        &mut imgui,
        None,
    ).expect("Failed to initialize ImGui Vulkan renderer");

    println!("Editor initialized successfully!");
    println!("Window: {:?}", window.id());
    println!("Swapchain images: {}", swapchain.images.len());
    println!("Graphics queue family: {}", context.queue_family_indices[0]);

    let mut frame_count = 0u32;
    let mut last_frame = Instant::now();

    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent { event: window_event, .. } => match window_event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                WindowEvent::RedrawRequested => {
                    // Update delta time
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                    
                    // Prepare ImGui frame
                    platform.prepare_frame(imgui.io_mut(), window.as_ref()).ok();
                    
                    // Build ImGui UI
                    {
                        let ui = imgui.frame();
                        ui.window("ImGui Demo").build(|| {
                            ui.text("Vulkan Backend is Working!");
                            ui.separator();
                            ui.text(format!("Frame: {}", frame_count));
                            ui.text("ImGui Text Rendering Coming Soon!");
                            if ui.button("Click me!") {
                                println!("Button clicked!");
                            }
                        });
                    }
                    
                    // Render ImGui to get draw data
                    let draw_data = imgui.render();
                    
                    // Get current frame index and synchronization primitives
                    let frame_index = frame_sync.current_frame;
                    let image_available = frame_sync.image_available_semaphores[frame_index];
                    let render_finished = frame_sync.render_finished_semaphores[frame_index];
                    let in_flight_fence = frame_sync.in_flight_fences[frame_index];
                    
                    // Wait for previous frame to complete
                    unsafe {
                        context.device.wait_for_fences(&[in_flight_fence], true, u64::MAX)
                            .expect("Failed to wait for fence");
                        context.device.reset_fences(&[in_flight_fence])
                            .expect("Failed to reset fence");
                    }
                    
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
                        Err(e) => {
                            eprintln!("Failed to acquire next image: {:?}", e);
                            return;
                        }
                    };
                    
                    // Get command buffer for this frame
                    let cmd_buffer = command_pool.buffers[frame_index];
                    
                    // Reset and begin recording command buffer
                    unsafe {
                        context.device.reset_command_buffer(
                            cmd_buffer,
                            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                        ).expect("Failed to reset command buffer");
                        
                        let begin_info = vk::CommandBufferBeginInfo::default()
                            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
                        context.device.begin_command_buffer(cmd_buffer, &begin_info)
                            .expect("Failed to begin command buffer");
                    }
                    
                    // Transition image to attachment optimal
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
                    
                    // Begin dynamic rendering
                    let mut color_attachment = DynamicRenderingAttachment::color(
                        swapchain.image_views[image_index as usize],
                        vk::AttachmentLoadOp::CLEAR,
                        vk::AttachmentStoreOp::STORE,
                    );
                    
                    // Set clear color to dark blue so we can see it's rendering
                    color_attachment.clear_value = vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.1, 0.2, 0.4, 1.0],
                        },
                    };
                    
                    unsafe {
                        let color_attachment_info = vk::RenderingAttachmentInfo::default()
                            .image_view(color_attachment.image_view)
                            .image_layout(color_attachment.image_layout)
                            .load_op(color_attachment.load_op)
                            .store_op(color_attachment.store_op)
                            .clear_value(color_attachment.clear_value);
                        
                        let rendering_info = vk::RenderingInfo::default()
                            .render_area(
                                vk::Rect2D::default()
                                    .extent(swapchain.extent),
                            )
                            .layer_count(1)
                            .color_attachments(std::slice::from_ref(&color_attachment_info));
                        
                        context.device.cmd_begin_rendering(cmd_buffer, &rendering_info);
                        
                        // Draw a simple triangle (white)
                        let viewport = vk::Viewport::default()
                            .x(0.0)
                            .y(0.0)
                            .width(swapchain.extent.width as f32)
                            .height(swapchain.extent.height as f32)
                            .min_depth(0.0)
                            .max_depth(1.0);
                        context.device.cmd_set_viewport(cmd_buffer, 0, &[viewport]);
                        
                        let scissor = vk::Rect2D::default()
                            .offset(vk::Offset2D { x: 0, y: 0 })
                            .extent(swapchain.extent);
                        context.device.cmd_set_scissor(cmd_buffer, 0, &[scissor]);
                        
                        context.device.cmd_end_rendering(cmd_buffer);
                    }
                    
                    // Render ImGui on top using traditional render pass
                    unsafe {
                        let framebuffer = context.device.create_framebuffer(
                            &vk::FramebufferCreateInfo::default()
                                .render_pass(render_pass)
                                .attachments(&[swapchain.image_views[image_index as usize]])
                                .width(swapchain.extent.width)
                                .height(swapchain.extent.height)
                                .layers(1),
                            None,
                        ).expect("Failed to create framebuffer");
                        
                        let clear_value = vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.1, 0.2, 0.4, 1.0],
                            },
                        };
                        let clear_values = [clear_value];
                        
                        let render_pass_info = vk::RenderPassBeginInfo::default()
                            .render_pass(render_pass)
                            .framebuffer(framebuffer)
                            .render_area(
                                vk::Rect2D::default()
                                    .extent(swapchain.extent),
                            )
                            .clear_values(&clear_values);
                        
                        context.device.cmd_begin_render_pass(cmd_buffer, &render_pass_info, vk::SubpassContents::INLINE);
                        
                        // Render ImGui draw data
                        renderer.cmd_draw(cmd_buffer, draw_data)
                            .expect("Failed to render ImGui");
                        
                        context.device.cmd_end_render_pass(cmd_buffer);
                        
                        context.device.destroy_framebuffer(framebuffer, None);
                    }
                    
                    // Transition image back for presentation
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
                        
                        context.device.end_command_buffer(cmd_buffer)
                            .expect("Failed to end command buffer");
                    }
                    
                    // Submit command buffer
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
                        
                        context.device.queue_submit(graphics_queue, &[submit_info], in_flight_fence)
                            .expect("Failed to submit to queue");
                    }
                    
                    // Present swapchain image
                    unsafe {
                        let render_finished_semaphores = [render_finished];
                        let swapchains = [swapchain.swapchain];
                        let image_indices = [image_index];
                        let present_info = vk::PresentInfoKHR::default()
                            .wait_semaphores(&render_finished_semaphores)
                            .swapchains(&swapchains)
                            .image_indices(&image_indices);
                        
                        let _ = swapchain_loader.queue_present(graphics_queue, &present_info);
                    }
                    
                    // Advance to next frame
                    frame_sync.current_frame = (frame_sync.current_frame + 1) % frame_sync.max_frames_in_flight;
                    frame_count += 1;
                    
                    if frame_count % 60 == 0 {
                        println!("Frames rendered: {}", frame_count);
                    }
                }
                _ => {
                    // platform.handle_event can only handle Event, but we have WindowEvent here
                    // We'll just skip it for now - focus on Vulkan rendering
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}