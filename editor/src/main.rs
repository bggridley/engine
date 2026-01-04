use anyhow::Result;
use engine::{
    gui::{ButtonComponent, PanelComponent, ContainerPanel, ComponentRef, UISystem, LayoutSpec, SizeSpec, HAlign, VAlign, TextComponent},
    renderer::{Renderer, VulkanContext, FontAtlas},
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

    let window_size = window.inner_size();
    println!(
        "Window physical size: {}x{} (DPI scale factor: {:.2})",
        window_size.width,
        window_size.height,
        window.scale_factor()
    );

    let context = Arc::new(VulkanContext::new(window.clone())?);
    let mut renderer = Some(Renderer::new(context.clone(), window_size.width, window_size.height)?);

    // Get the shared descriptor_set_layout for text rendering from the pipeline manager
    // This avoids creating redundant layouts - all TextComponents share this one layout
    let text_descriptor_layout = renderer.as_ref().unwrap()
        .get_descriptor_set_layout(engine::renderer::PipelineId::Text)
        .expect("Text pipeline should have descriptor_set_layout");

    // Load font atlas at exact target font size
    let font_atlas: Arc<FontAtlas> = Arc::new(FontAtlas::load(
        "./assets/segoeui.ttf",
        18.0,  // Target font size for pixel-perfect rendering
        &context.device,
        &context.instance,
        context.physical_device,
        context.queue_family_indices[0],
    )?);

    //Entity1thisissometext

    let mut ui = UISystem::new();

    // === MENU BAR (File, Edit, View, Help) ===
    let menu_row = ui.grid.add_row();
    let mut menu_container = ContainerPanel::new(&context, [0.08, 0.08, 0.12])?;
    
    // Create a single row in the menu container for horizontal layout
    let menu_items_row = menu_container.grid_mut().add_row();
    
    // Create menu buttons
    let mut file_button = ButtonComponent::new(&context)?;
    file_button.set_text(TextComponent::new("File", font_atlas.clone(), 18.0, text_descriptor_layout, &context)?);
    
    let mut edit_button = ButtonComponent::new(&context)?;
    edit_button.set_text(TextComponent::new("Edit", font_atlas.clone(), 18.0, text_descriptor_layout, &context)?);
    
    let mut view_button = ButtonComponent::new(&context)?;
    view_button.set_text(TextComponent::new("View", font_atlas.clone(), 18.0, text_descriptor_layout, &context)?);
    
    let mut help_button = ButtonComponent::new(&context)?;
    help_button.set_text(TextComponent::new("Help", font_atlas.clone(), 18.0, text_descriptor_layout, &context)?);
    
    // Menu button spec
    let menu_button_spec = LayoutSpec::new(SizeSpec::Fixed(70.0), SizeSpec::Percent(1.0))
        .with_alignment(HAlign::Left, VAlign::Middle)
        .with_padding(0.0)
        .with_margin(0.0);
    
    menu_container.grid_mut().get_row_mut(menu_items_row).unwrap().add_component(Box::new(file_button), menu_button_spec);
    menu_container.grid_mut().get_row_mut(menu_items_row).unwrap().add_component(Box::new(edit_button), menu_button_spec);
    menu_container.grid_mut().get_row_mut(menu_items_row).unwrap().add_component(Box::new(view_button), menu_button_spec);
    menu_container.grid_mut().get_row_mut(menu_items_row).unwrap().add_component(Box::new(help_button), menu_button_spec);
    
    // Wrap menu container
    let (menu_wrapper, menu_handle) = ComponentRef::new(menu_container);
    
    let menu_spec = LayoutSpec::new(SizeSpec::Percent(1.0), SizeSpec::Fixed(18.0))
        .with_alignment(HAlign::Left, VAlign::Top);
    ui.grid.get_row_mut(menu_row).unwrap().add_component(Box::new(menu_wrapper), menu_spec);

    // === MAIN ROW: Left sidebar + Right content ===
    let main_row = ui.grid.add_row();

    // LEFT SIDEBAR CONTAINER (takes ~20% width)
    let mut left_container = ContainerPanel::new(&context, [0.15, 0.15, 0.2])?;
    
    // Add 3 rows to the sidebar container for ECS items
    let sidebar_row1 = left_container.grid_mut().add_row();
    let sidebar_row2 = left_container.grid_mut().add_row();
    let sidebar_row3 = left_container.grid_mut().add_row();

    // Create ECS buttons with text
    let mut ecs_button1 = ButtonComponent::new(&context)?;
    ecs_button1.set_text(TextComponent::new("FPS: 0.0", font_atlas.clone(), 18.0, text_descriptor_layout, &context)?);
    let (fps_button_wrapper, fps_button) = ComponentRef::new(ecs_button1);
    
    let mut ecs_button2 = ButtonComponent::new(&context)?;
    ecs_button2.set_text(TextComponent::new("Entity 2", font_atlas.clone(), 18.0, text_descriptor_layout, &context)?);

    let mut ecs_button3 = ButtonComponent::new(&context)?;
    ecs_button3.set_text(TextComponent::new("Entity 3", font_atlas.clone(), 18.0, text_descriptor_layout, &context)?);

    // Add buttons to sidebar rows
    let button_spec = LayoutSpec::new(SizeSpec::Percent(1.0), SizeSpec::Fixed(30.0))
        .with_alignment(HAlign::Center, VAlign::Top)
        .with_padding(0.0)
        .with_margin(0.0);
    
    left_container.grid_mut().get_row_mut(sidebar_row1).unwrap().add_component(Box::new(fps_button_wrapper), button_spec);
    left_container.grid_mut().get_row_mut(sidebar_row2).unwrap().add_component(Box::new(ecs_button2), button_spec);
    left_container.grid_mut().get_row_mut(sidebar_row3).unwrap().add_component(Box::new(ecs_button3), button_spec);

    // Wrap container for external access
    let (container_wrapper, container_handle) = ComponentRef::new(left_container);
    
    // Add left container to main row
    let left_container_spec = LayoutSpec::new(SizeSpec::Percent(0.15), SizeSpec::Percent(1.0))
        .with_alignment(HAlign::Left, VAlign::Middle);
    
    ui.grid.get_row_mut(main_row).unwrap().add_component(
        Box::new(container_wrapper),
        left_container_spec,
    );

    // RIGHT CONTENT PANEL (takes ~80% width)
    let right_panel = PanelComponent::new(&context, [0.3, 0.3, 0.35])?;
    let right_panel_spec = LayoutSpec::new(SizeSpec::Percent(1.0), SizeSpec::Percent(1.0))
        .with_alignment(HAlign::Center, VAlign::Middle);
    ui.grid.get_row_mut(main_row).unwrap().add_component(Box::new(right_panel), right_panel_spec);

    // Set initial bounds
    ui.grid.set_bounds(0.0, 0.0, window_size.width as f32, window_size.height as f32);
    
    // Update nested container layouts
    menu_handle.borrow_mut().update_grid_layout();
    container_handle.borrow_mut().update_grid_layout();

    println!("Vulkan Engine initialized!");

    let mut frame_count = 0u32;
    let mut last_resize_size: Option<(u32, u32)> = None;
    let mut mouse_pos = (0.0f32, 0.0f32);
    
    // FPS tracking
    let mut last_fps_update = std::time::Instant::now();
    let mut fps_frame_count = 0u32;
    let mut current_fps = 0.0f32;

    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => {
                    // Clean up GPU resources in proper order before exiting
                    unsafe { context.device.device_wait_idle().ok(); }
                    ui.destroy(&context.device);
                    font_atlas.destroy(&context.device);
                    if let Some(r) = renderer.take() {
                        drop(r);
                    }
                    window_target.exit();
                }
                WindowEvent::Resized(new_size) => {
                    last_resize_size = Some((new_size.width, new_size.height));
                    window.request_redraw();
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let window_size = window.inner_size();
                    let inverted_y = window_size.height as f32 - position.y as f32;
                    mouse_pos = (position.x as f32, inverted_y);
                    ui.handle_mouse_move(position.x as f32, inverted_y);
                    window.request_redraw();
                }

                WindowEvent::MouseInput { state, .. } => match state {
                    winit::event::ElementState::Pressed => {
                        ui.handle_mouse_down(mouse_pos.0, mouse_pos.1);
                        window.request_redraw();
                    }

                    winit::event::ElementState::Released => {
                        ui.handle_mouse_up(mouse_pos.0, mouse_pos.1);
                        window.request_redraw();
                    }
                },

                WindowEvent::RedrawRequested => {
                    // Handle resize
                    if let Some((width, height)) = last_resize_size.take() {
                        if let Some(ref mut r) = renderer {
                            r.handle_resize(width, height, window.scale_factor() as f32);
                        }
                        ui.grid.set_bounds(0.0, 0.0, width as f32, height as f32);
                        menu_handle.borrow_mut().update_grid_layout();
                        container_handle.borrow_mut().update_grid_layout();
                    }
                    
                    // Update FPS counter
                    fps_frame_count += 1;
                    let elapsed = last_fps_update.elapsed();
                    if elapsed.as_secs_f32() >= 0.5 {
                        current_fps = fps_frame_count as f32 / elapsed.as_secs_f32();
                        fps_frame_count = 0;
                        last_fps_update = std::time::Instant::now();
                        
                        // Update FPS button text
                        let fps_text = format!("FPS: {:.1}", current_fps);
                        fps_button.borrow_mut().update_text(&fps_text, &context).ok();
                    }

                    // Begin frame and render
                    if let Some(ref mut r) = renderer {
                        if let Some(frame) = r.begin_frame() {
                            ui.render(&frame.render_ctx, r).ok();

                            frame_count += 1;
                            if frame_count % 60 == 0 {
                                println!("Frames: {} | FPS: {:.1}", frame_count, current_fps);
                            }
                        }
                    }
                }
                _ => {}
            },

            //Event::AboutToWait => {
            //    window.request_redraw();
            //}            
            // Remove continuous rendering - only redraw on demand
            // Later: add a flag here for continuous mode when game preview is active
            _ => {}
        }
    })?;

    Ok(())
}
