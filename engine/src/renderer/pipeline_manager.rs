use ash::{vk, Device};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::{PipelineBuilder, ShaderId};

/// Predefined pipeline types in the engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum PipelineId {
    /// Basic colored triangle/geometry rendering
    BasicGeometry,
    /// UI rendering with alpha blending
    UI,
}

/// Static metadata for pipeline configuration
struct PipelineMeta {
    vertex_shader: ShaderId,
    fragment_shader: ShaderId,
    blend_enabled: bool,
    cull_mode: vk::CullModeFlags,
}

impl PipelineId {
    fn meta(&self) -> PipelineMeta {
        match self {
            PipelineId::BasicGeometry => PipelineMeta {
                vertex_shader: ShaderId::TriangleVertex,
                fragment_shader: ShaderId::TriangleFrag,
                blend_enabled: false,
                cull_mode: vk::CullModeFlags::BACK,
            },
            PipelineId::UI => PipelineMeta {
                vertex_shader: ShaderId::TriangleVertex,
                fragment_shader: ShaderId::TriangleFrag,
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

        // Vertex input for basic colored vertices (position + color)
        let vertex_bindings = vec![
            vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<crate::renderer::Vertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX),
        ];

        let vertex_attributes = vec![
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(8),
        ];

        PipelineBuilder::new(vert_code, frag_code)
            .vertex_input(vertex_bindings, vertex_attributes)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(meta.cull_mode, vk::FrontFace::COUNTER_CLOCKWISE)
            .color_format(vk::Format::B8G8R8A8_SRGB)
            .blending(meta.blend_enabled)
            .build(device)
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
