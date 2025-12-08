use anyhow::Result;
use engine::{
    window::EventLoop,
    renderer::{VulkanContext, Swapchain, CommandPool, FrameSynchronizer, RenderContext},
    gui::{UISystem, TriangleComponent},
};
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
        vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_SRGB,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        },
        vk::Extent2D {
            width: 1280,
            height: 720,
        },
        context.surface,
        vk::PresentModeKHR::FIFO,
        2,
        &context.queue_family_indices,
    );

    // Create command pool and frame synchronizer
    let command_pool = CommandPool::new(&context.device, context.queue_family_indices[0], 2);
    let mut frame_sync = FrameSynchronizer::new(&context.device, 2);

    // Get graphics queue
    let graphics_queue = unsafe {
        context.device.get_device_queue(context.queue_family_indices[0], 0)
    };

    // Create UI system
    let mut ui = UISystem::new();
    let triangle = TriangleComponent::new(&context)?;
    ui.add_component(Box::new(triangle));

    println!("Vulkan Engine initialized!");

    let mut frame_count = 0u32;

    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent { event: window_event, .. } => match window_event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                WindowEvent::RedrawRequested => {
                    frame_sync.begin_frame().ok();

                    let image_index = match unsafe {
                        swapchain_loader.acquire_next_image(
                            swapchain.swapchain,
                            u64::MAX,
                            frame_sync.current_image_available_semaphore(),
                            vk::Fence::null(),
                        )
                    } {
                        Ok((idx, _)) => idx,
                        Err(_) => return,
                    };

                    let cmd_buffer = command_pool.buffers[frame_sync.current_frame_index()];

                    // Record commands
                    unsafe {
                        context.device.reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).ok();
                        let begin_info = vk::CommandBufferBeginInfo::default()
                            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
                        context.device.begin_command_buffer(cmd_buffer, &begin_info).ok();
                    }

                    let render_ctx = RenderContext::new(
                        context.device.clone(),
                        cmd_buffer,
                        swapchain.extent,
                    );

                    // Transition to render target
                    render_ctx.transition_image(
                        swapchain.images[image_index as usize],
                        vk::ImageLayout::UNDEFINED,
                        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    );

                    // Begin rendering
                    render_ctx.begin_rendering(
                        swapchain.image_views[image_index as usize],
                        [0.25, 0.1, 0.1, 1.0],
                    );

                    // Render UI
                    ui.render(&render_ctx).ok();

                    // End rendering
                    render_ctx.end_rendering();

                    // Transition to present
                    render_ctx.transition_image(
                        swapchain.images[image_index as usize],
                        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        vk::ImageLayout::PRESENT_SRC_KHR,
                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    );

                    unsafe {
                        context.device.end_command_buffer(cmd_buffer).ok();
                    }

                    // Submit and present
                    unsafe {
                        let cmd_buffers = [cmd_buffer];
                        let wait_sems = [frame_sync.current_image_available_semaphore()];
                        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                        let signal_sems = [frame_sync.current_render_finished_semaphore()];

                        let submit_info = vk::SubmitInfo::default()
                            .command_buffers(&cmd_buffers)
                            .wait_semaphores(&wait_sems)
                            .wait_dst_stage_mask(&wait_stages)
                            .signal_semaphores(&signal_sems);

                        context.device.queue_submit(graphics_queue, &[submit_info], frame_sync.current_in_flight_fence()).ok();

                        let signal_sems_present = [frame_sync.current_render_finished_semaphore()];
                        let swapchains = [swapchain.swapchain];
                        let image_indices = [image_index];

                        let present_info = vk::PresentInfoKHR::default()
                            .wait_semaphores(&signal_sems_present)
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