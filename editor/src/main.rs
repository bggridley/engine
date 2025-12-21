use anyhow::Result;
use engine::{
    gui::{GUIComponent, TriangleComponent, UISystem}, renderer::{Renderer, VulkanContext}, window::EventLoop
};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = WindowBuilder::new()
        .with_title("Vulkan Engine")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)?;
    let window = Arc::new(window);

    // Get actual physical size for DPI awareness
    let window_size = window.inner_size();
    println!("Window physical size: {}x{} (DPI scale factor: {:.2})", 
        window_size.width, window_size.height, window.scale_factor());

    // Initialize Vulkan
    let context = Arc::new(VulkanContext::new(window.clone())?);
    let mut renderer = Renderer::new(context.clone(), window_size.width, window_size.height)?;

    // Create UI system
    let mut ui = UISystem::new();
    let triangle = TriangleComponent::new(&context)?;
    ui.add_component(Box::new(triangle));

    println!("Vulkan Engine initialized!");

    let mut frame_count = 0u32;
    let mut last_resize_size: Option<(u32, u32)> = None;

    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent { event: window_event, .. } => match window_event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                WindowEvent::Resized(new_size) => {
                    last_resize_size = Some((new_size.width, new_size.height));
                }
                WindowEvent::RedrawRequested => {
                    // Handle resize
                    if let Some((width, height)) = last_resize_size.take() {
                        renderer.handle_resize(width, height, window.scale_factor() as f32);
                    }

                    // Begin frame and render
                    if let Some(frame) = renderer.begin_frame() {
                        ui.render(&frame.render_ctx, &mut renderer).ok();
                        // Frame automatically ends when dropped
                        
                        frame_count += 1;
                        if frame_count % 60 == 0 {
                            println!("Frames: {}", frame_count);
                        }
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            },
            _ => {}
        }
    })?;

    Ok(())
}