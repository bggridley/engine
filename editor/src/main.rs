use engine::window::EventLoop; // re-exported from winit
use engine::window::App;
use anyhow::Result;

fn main() -> Result<()> {

    let event_loop = EventLoop::new()?;
    let mut app = App::default();

    event_loop.run_app(&mut app)?;

    Ok(())
}