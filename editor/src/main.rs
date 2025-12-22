use anyhow::Result;
use engine::{
    gui::{ButtonComponent, PanelComponent, ContainerPanel, GUIComponent, UISystem, LayoutSpec, SizeSpec, HAlign, VAlign, Transform2D},
    renderer::{Renderer, VulkanContext, RenderContext},
    window::EventLoop,
};
use std::sync::Arc;
use std::cell::RefCell;
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
    let mut renderer = Renderer::new(context.clone(), window_size.width, window_size.height)?;

    let mut ui = UISystem::new();

    // === MAIN ROW: Left sidebar + Right content ===
    let main_row = ui.grid.add_row();

    // LEFT SIDEBAR CONTAINER (takes ~20% width)
    let mut left_container = ContainerPanel::new(&context, [0.15, 0.15, 0.2])?;
    
    // Add 3 rows to the sidebar container for ECS items
    let sidebar_row1 = left_container.grid_mut().add_row();
    let sidebar_row2 = left_container.grid_mut().add_row();
    let sidebar_row3 = left_container.grid_mut().add_row();

    // Create ECS buttons
    let ecs_button1 = ButtonComponent::new(&context)?;
    let ecs_button2 = ButtonComponent::new(&context)?;
    let ecs_button3 = ButtonComponent::new(&context)?;

    // Add buttons to sidebar rows (one button per row, takes full width of that row)
    let button_spec = LayoutSpec::new(SizeSpec::Percent(1.0), SizeSpec::Fixed(30.0))
        .with_alignment(HAlign::Center, VAlign::Top)
        .with_padding(0.0)
        .with_margin(0.0);

    left_container.grid_mut().get_row_mut(sidebar_row1).unwrap().add_component(Box::new(ecs_button1), button_spec);
    left_container.grid_mut().get_row_mut(sidebar_row2).unwrap().add_component(Box::new(ecs_button2), button_spec);
    left_container.grid_mut().get_row_mut(sidebar_row3).unwrap().add_component(Box::new(ecs_button3), button_spec);

    // Wrap in RefCell to allow interior mutability for layout updates
    let left_container = RefCell::new(left_container);

    // Add left container to main row
    let left_container_spec = LayoutSpec::new(SizeSpec::Percent(0.15), SizeSpec::Percent(1.0))
        .with_alignment(HAlign::Left, VAlign::Middle);
    
    // We need a wrapper that implements GUIComponent and forwards to the RefCell
    struct ContainerWrapper {
        container: Arc<RefCell<ContainerPanel>>,
        cached_transform: Transform2D,
    }
    
    impl GUIComponent for ContainerWrapper {
        fn render(&self, ctx: &RenderContext, renderer: &mut Renderer) -> Result<()> {
            // Sync the cached transform with the container before rendering
            let mut container_mut = self.container.borrow_mut();
            *container_mut.transform_mut() = self.cached_transform;
            container_mut.update_grid_layout();
            container_mut.render(ctx, renderer)
        }
        fn transform(&self) -> &Transform2D {
            &self.cached_transform
        }
        fn transform_mut(&mut self) -> &mut Transform2D {
            &mut self.cached_transform
        }
        fn handle_mouse_down(&mut self, x: f32, y: f32) {
            self.container.borrow_mut().handle_mouse_down(x, y);
        }
        fn handle_mouse_up(&mut self, x: f32, y: f32) {
            self.container.borrow_mut().handle_mouse_up(x, y);
        }
        fn handle_mouse_move(&mut self, x: f32, y: f32) {
            self.container.borrow_mut().handle_mouse_move(x, y);
        }
    }
    
    let container_arc = Arc::new(left_container);
    let mut wrapper = ContainerWrapper {
        container: container_arc.clone(),
        cached_transform: Transform2D::new(),
    };
    
    ui.grid.get_row_mut(main_row).unwrap().add_component(
        Box::new(wrapper),
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
    container_arc.borrow_mut().update_grid_layout();

    println!("Vulkan Engine initialized!");

    let mut frame_count = 0u32;
    let mut last_resize_size: Option<(u32, u32)> = None;
    let mut mouse_pos = (0.0f32, 0.0f32);

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

                WindowEvent::MouseInput { state, .. } => match state {
                    winit::event::ElementState::Pressed => {
                        ui.handle_mouse_down(mouse_pos.0, mouse_pos.1);
                    }

                    winit::event::ElementState::Released => {
                        ui.handle_mouse_up(mouse_pos.0, mouse_pos.1);
                    }
                },

                WindowEvent::RedrawRequested => {
                    // Handle resize
                    if let Some((width, height)) = last_resize_size.take() {
                        renderer.handle_resize(width, height, window.scale_factor() as f32);
                        ui.grid.set_bounds(0.0, 0.0, width as f32, height as f32);
                        container_arc.borrow_mut().update_grid_layout();
                    }

                    // Begin frame and render
                    if let Some(frame) = renderer.begin_frame() {
                        ui.render(&frame.render_ctx, &mut renderer).ok();

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
