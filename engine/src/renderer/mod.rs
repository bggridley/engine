use anyhow::Result;
use std::sync::Arc;

use winit::window::Window;

mod context;
pub use context::VulkanContext;

pub mod swapchain;
pub use swapchain::Swapchain;

pub mod command_pool;
pub use command_pool::CommandPool;

pub mod sync;
pub use sync::FrameSynchronizer;

pub mod dynamic_rendering;
pub use dynamic_rendering::{DynamicRenderingAttachment, ViewportScissor, color_attachment, depth_attachment};

pub struct VulkanRenderer {}

impl VulkanRenderer {
    pub fn new(_window: Arc<Window>, _context: Arc<VulkanContext>) -> Result<Self> {
        Ok(Self {})
    }
}
