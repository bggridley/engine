use ash::{vk, Device};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::{PipelineBuilder, ShaderId, VertexFormat};

/// Predefined pipeline types in the engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum PipelineId {
    /// Basic colored triangle/geometry rendering
    BasicGeometry,
    /// UI rendering with alpha blending
    UI,
    /// Text rendering with font atlas
    Text,
}

/// Static metadata for pipeline configuration
struct PipelineMeta {
    vertex_shader: ShaderId,
    fragment_shader: ShaderId,
    vertex_format: VertexFormat,
    blend_enabled: bool,
    cull_mode: vk::CullModeFlags,
}

impl PipelineId {
    fn meta(&self) -> PipelineMeta {
        match self {
            PipelineId::BasicGeometry => PipelineMeta {
                vertex_shader: ShaderId::TriangleVertex,
                fragment_shader: ShaderId::TriangleFrag,
                vertex_format: VertexFormat::ColorVertex2D,
                blend_enabled: false,
                cull_mode: vk::CullModeFlags::BACK,
            },
            PipelineId::UI => PipelineMeta {
                vertex_shader: ShaderId::TriangleVertex,
                fragment_shader: ShaderId::TriangleFrag,
                vertex_format: VertexFormat::ColorVertex2D,
                blend_enabled: true,
                cull_mode: vk::CullModeFlags::NONE,
            },
            PipelineId::Text => PipelineMeta {
                vertex_shader: ShaderId::TextVertex,
                fragment_shader: ShaderId::TextFrag,
                vertex_format: VertexFormat::TexturedVertex2D,
                blend_enabled: true,
                cull_mode: vk::CullModeFlags::NONE,
            },
        }
    }

    /// Build the pipeline from metadata
    pub fn build(&self, device: &Arc<Device>) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
        let meta = self.meta();
        
        let vert_code = meta.vertex_shader.load_shader_bytes()?;
        let frag_code = meta.fragment_shader.load_shader_bytes()?;

        // Get vertex format from metadata
        let vertex_bindings = vec![meta.vertex_format.binding()];
        let vertex_attributes = meta.vertex_format.attributes();

        let mut builder = PipelineBuilder::new(vert_code, frag_code)
            .vertex_input(vertex_bindings, vertex_attributes)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(meta.cull_mode, vk::FrontFace::COUNTER_CLOCKWISE)
            .color_format(vk::Format::B8G8R8A8_SRGB)
            .blending(meta.blend_enabled);

        // Add descriptor sets for Text pipeline texture sampling
        if *self == PipelineId::Text {
            let bindings = vec![
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ];
            
            let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&bindings);
            
            let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
            builder = builder.descriptor_set_layouts(vec![descriptor_set_layout]);
        }

        builder.build(device)
    }

    pub fn all() -> impl Iterator<Item = PipelineId> {
        PipelineId::iter()
    }
}

/// Manages all graphics pipelines with enum-based access
pub struct PipelineManager {
    device: Arc<Device>,
    pipelines: HashMap<PipelineId, vk::Pipeline>,
    layouts: HashMap<PipelineId, vk::PipelineLayout>,
}

impl PipelineManager {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            pipelines: HashMap::new(),
            layouts: HashMap::new(),
        }
    }

    /// Build and cache all pipelines
    pub fn build_all(&mut self) -> Result<()> {
        for id in PipelineId::all() {
            self.build_pipeline(id)?;
        }
        Ok(())
    }

    /// Build a specific pipeline
    fn build_pipeline(&mut self, id: PipelineId) -> Result<()> {
        if self.pipelines.contains_key(&id) {
            return Ok(());
        }

        let (pipeline, layout) = id.build(&self.device)?;
        self.pipelines.insert(id, pipeline);
        self.layouts.insert(id, layout);

        Ok(())
    }

    /// Get a pipeline (builds it if not cached)
    pub fn get(&mut self, id: PipelineId) -> Result<vk::Pipeline> {
        if !self.pipelines.contains_key(&id) {
            self.build_pipeline(id)?;
        }
        Ok(self.pipelines[&id])
    }

    /// Get a pipeline layout
    pub fn get_layout(&self, id: PipelineId) -> Option<vk::PipelineLayout> {
        self.layouts.get(&id).copied()
    }
}

impl Drop for PipelineManager {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            
            for &pipeline in self.pipelines.values() {
                self.device.destroy_pipeline(pipeline, None);
            }
            for &layout in self.layouts.values() {
                self.device.destroy_pipeline_layout(layout, None);
            }
        }
    }
}
