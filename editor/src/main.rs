use anyhow::Result;
use engine::{
    gui::{GUIComponent, ComponentHandle, ButtonComponent, UISystem, Vec2},
    renderer::{Renderer, VulkanContext},
    window::EventLoop,
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
    println!(
        "Window physical size: {}x{} (DPI scale factor: {:.2})",
        window_size.width,
        window_size.height,
        window.scale_factor()
    );

    // Initialize Vulkan
    let context = Arc::new(VulkanContext::new(window.clone())?);
    let mut renderer = Renderer::new(context.clone(), window_size.width, window_size.height)?;

    // Create UI system
    let mut ui = UISystem::new();
    let mut button = ButtonComponent::new(&context)?;

    button.transform_mut().scale = Vec2::new(100.0, 50.0);
    button.transform_mut().position = Vec2::new(200.0, 200.0);
    // let mut offset = 0.0;
    let _: ComponentHandle = ui.add_component(Box::new(button));

    println!("Vulkan Engine initialized!");

    let mut frame_count = 0u32;
    let mut last_resize_size: Option<(u32, u32)> = None;
    let mut mouse_pos = (0.0f32, 0.0f32);  // Track mouse position

    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => {
                    window_target.exit();
                }

                WindowEvent::Resized(new_size) => {
                    last_resize_size = Some((new_size.width, new_size.height));
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let window_size = window.inner_size();
                    let inverted_y = window_size.height as f32 - position.y as f32;
                    mouse_pos = (position.x as f32, inverted_y);
                    ui.handle_mouse_move(position.x as f32, inverted_y);
                }

                WindowEvent::MouseInput { state, .. } => {
                    match state {
                        winit::event::ElementState::Pressed => {
                            ui.handle_mouse_down(mouse_pos.0, mouse_pos.1);
                        }

                        winit::event::ElementState::Released => {
                            ui.handle_mouse_up(mouse_pos.0, mouse_pos.1);
                        }
                    }
                }

                WindowEvent::RedrawRequested => {
                    // Handle resize
                    if let Some((width, height)) = last_resize_size.take() {
                        renderer.handle_resize(width, height, window.scale_factor() as f32);
                    }

                    // if let Some(triangle) = ui.get_component_mut(&triangle_handle) {
                    //     offset += 1.0 / (2.0 * 3.14);
                    //     triangle.set_rotation(offset);
                    // }

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
            }
            _ => {}
        }
    })?;

    Ok(())
}
