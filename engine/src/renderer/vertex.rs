use ash::vk;

/// 2D vertex with RGB color
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ColorVertex2D {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

/// 2D vertex with UV texture coordinates
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TexturedVertex2D {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

/// 3D vertex for models with normal and UV
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelVertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Push constants for rendering (projection + transform matrices + color modulation)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PushConstants2D {
    pub projection: glam::Mat4,
    pub transform: glam::Mat4,
    pub color_modulation: [f32; 3],  // RGB multiplier (e.g., [0.8, 0.8, 0.8] = 20% darker)
    pub _padding: f32,  // Align to 16 bytes for uniform buffer rules
}

/// Vertex format descriptor for pipeline creation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexFormat {
    ColorVertex2D,
    TexturedVertex2D,
    ModelVertex3D,
}

impl VertexFormat {
    /// Get vertex binding description
    pub fn binding(&self) -> vk::VertexInputBindingDescription {
        match self {
            VertexFormat::ColorVertex2D => vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<ColorVertex2D>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            VertexFormat::TexturedVertex2D => vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<TexturedVertex2D>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            VertexFormat::ModelVertex3D => vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<ModelVertex3D>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            },
        }
    }

    /// Get vertex attribute descriptions
    pub fn attributes(&self) -> Vec<vk::VertexInputAttributeDescription> {
        match self {
            VertexFormat::ColorVertex2D => vec![
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: 0,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: 8,
                },
            ],
            VertexFormat::TexturedVertex2D => vec![
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: 0,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: 8,
                },
            ],
            VertexFormat::ModelVertex3D => vec![
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: 0,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
                    offset: 12,
                },
                vk::VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: 24,
                },
            ],
        }
    }
}
