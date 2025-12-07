use ash::vk;

/// Helper for setting up Vulkan 1.3 dynamic rendering
pub struct DynamicRenderingAttachment {
    pub image_view: vk::ImageView,
    pub image_layout: vk::ImageLayout,
    pub load_op: vk::AttachmentLoadOp,
    pub store_op: vk::AttachmentStoreOp,
    pub clear_value: vk::ClearValue,
}

impl DynamicRenderingAttachment {
    /// Create a rendering attachment info for a color attachment
    pub fn color(
        image_view: vk::ImageView,
        load_op: vk::AttachmentLoadOp,
        store_op: vk::AttachmentStoreOp,
    ) -> Self {
        Self {
            image_view,
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            load_op,
            store_op,
            clear_value: vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
        }
    }

    /// Create a rendering attachment info for a depth attachment
    pub fn depth(
        image_view: vk::ImageView,
        load_op: vk::AttachmentLoadOp,
        store_op: vk::AttachmentStoreOp,
    ) -> Self {
        Self {
            image_view,
            image_layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            load_op,
            store_op,
            clear_value: vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        }
    }

    /// Set custom clear color
    pub fn with_clear_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [r, g, b, a],
            },
        };
        self
    }
}

/// Helper to create rendering attachment info for color targets
pub fn color_attachment(
    image_view: vk::ImageView,
    load_op: vk::AttachmentLoadOp,
    store_op: vk::AttachmentStoreOp,
) -> DynamicRenderingAttachment {
    DynamicRenderingAttachment::color(image_view, load_op, store_op)
}

/// Helper to create rendering attachment info for depth targets
pub fn depth_attachment(
    image_view: vk::ImageView,
    load_op: vk::AttachmentLoadOp,
    store_op: vk::AttachmentStoreOp,
) -> DynamicRenderingAttachment {
    DynamicRenderingAttachment::depth(image_view, load_op, store_op)
}

/// Viewport and scissor helper
pub struct ViewportScissor {
    pub viewports: Vec<vk::Viewport>,
    pub scissors: Vec<vk::Rect2D>,
}

impl ViewportScissor {
    /// Create a viewport and scissor for the given extent
    pub fn new(width: f32, height: f32) -> Self {
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: width as u32,
                height: height as u32,
            },
        };

        Self {
            viewports: vec![viewport],
            scissors: vec![scissor],
        }
    }
}
