
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

pub use winit::event_loop::EventLoop;
pub use handler::WindowHandler; // re-export this module
pub mod handler;

#[derive(Default)]
pub struct App {
    window_handler: Option<WindowHandler>
}

impl ApplicationHandler for App {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_handler = Some(WindowHandler::new(event_loop, WindowAttributes::default()).unwrap());
        
        if let Some(handler) = self.window_handler.as_mut() {
            let secondary_window = handler.create_window(event_loop, WindowAttributes::default().with_title("deez nuts")).unwrap();
        }
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(window_handler) = self.window_handler.as_mut() {
            window_handler.window_event(event_loop, window_id, event);
        }
    }
}