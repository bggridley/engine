use anyhow::Result;
use ash::vk;
use std::sync::Arc;

/// Generic vertex buffer that can hold any vertex type
pub struct VertexBuffer<V> {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub vertex_count: u32,
    device: Arc<ash::Device>,
    _phantom: std::marker::PhantomData<V>,
}

impl<V> VertexBuffer<V> {
    /// Create a vertex buffer from vertex data
    pub fn new(
        device: &Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
        vertices: &[V],
    ) -> Result<Self> {
        let buffer_size = std::mem::size_of_val(vertices) as vk::DeviceSize;
        
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        // Find suitable memory type
        let mem_type_index = find_memory_type(
            instance,
            physical_device,
            &mem_requirements,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let memory = unsafe { device.allocate_memory(&alloc_info, None)? };

        unsafe {
            device.bind_buffer_memory(buffer, memory, 0)?;
        }

        // Copy vertex data to buffer
        unsafe {
            let data_ptr = device.map_memory(
                memory,
                0,
                mem_requirements.size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const u8,
                data_ptr as *mut u8,
                std::mem::size_of_val(vertices),
            );
            device.unmap_memory(memory);
        }

        Ok(VertexBuffer {
            buffer,
            memory,
            vertex_count: vertices.len() as u32,
            device: device.clone(),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<V> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

/// Generic index buffer
pub struct IndexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub index_count: u32,
    device: Arc<ash::Device>,
}

impl IndexBuffer {
    pub fn new(
        device: &Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
        indices: &[u32],
    ) -> Result<Self> {
        let buffer_size = std::mem::size_of_val(indices) as vk::DeviceSize;
        
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let mem_type_index = find_memory_type(
            instance,
            physical_device,
            &mem_requirements,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let memory = unsafe { device.allocate_memory(&alloc_info, None)? };

        unsafe {
            device.bind_buffer_memory(buffer, memory, 0)?;
        }

        unsafe {
            let data_ptr = device.map_memory(
                memory,
                0,
                mem_requirements.size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                indices.as_ptr() as *const u8,
                data_ptr as *mut u8,
                std::mem::size_of_val(indices),
            );
            device.unmap_memory(memory);
        }

        Ok(IndexBuffer {
            buffer,
            memory,
            index_count: indices.len() as u32,
            device: device.clone(),
        })
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

/// Generic pipeline builder for creating graphics pipelines with dynamic rendering
pub struct PipelineBuilder {
    vertex_shader: Vec<u32>,
    fragment_shader: Vec<u32>,
    vertex_bindings: Vec<vk::VertexInputBindingDescription>,
    vertex_attributes: Vec<vk::VertexInputAttributeDescription>,
    topology: vk::PrimitiveTopology,
    polygon_mode: vk::PolygonMode,
    cull_mode: vk::CullModeFlags,
    front_face: vk::FrontFace,
    color_format: vk::Format,
    enable_blending: bool,
}

impl PipelineBuilder {
    pub fn new(vertex_shader: Vec<u32>, fragment_shader: Vec<u32>) -> Self {
        Self {
            vertex_shader,
            fragment_shader,
            vertex_bindings: Vec::new(),
            vertex_attributes: Vec::new(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            color_format: vk::Format::B8G8R8A8_SRGB,
            enable_blending: false,
        }
    }

    pub fn vertex_input(
        mut self,
        bindings: Vec<vk::VertexInputBindingDescription>,
        attributes: Vec<vk::VertexInputAttributeDescription>,
    ) -> Self {
        self.vertex_bindings = bindings;
        self.vertex_attributes = attributes;
        self
    }

    pub fn topology(mut self, topology: vk::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn polygon_mode(mut self, mode: vk::PolygonMode) -> Self {
        self.polygon_mode = mode;
        self
    }

    pub fn cull_mode(mut self, mode: vk::CullModeFlags, front_face: vk::FrontFace) -> Self {
        self.cull_mode = mode;
        self.front_face = front_face;
        self
    }

    pub fn color_format(mut self, format: vk::Format) -> Self {
        self.color_format = format;
        self
    }

    pub fn blending(mut self, enable: bool) -> Self {
        self.enable_blending = enable;
        self
    }

    pub fn build(
        self,
        device: &Arc<ash::Device>,
    ) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
        // Create shader modules
        let vert_module = unsafe {
            device.create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&self.vertex_shader),
                None,
            )?
        };

        let frag_module = unsafe {
            device.create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&self.fragment_shader),
                None,
            )?
        };

        // Create pipeline layout
        let pipeline_layout = unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default(),
                None,
            )?
        };

        // Vertex input state
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&self.vertex_bindings)
            .vertex_attribute_descriptions(&self.vertex_attributes);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(self.topology);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(self.polygon_mode)
            .cull_mode(self.cull_mode)
            .front_face(self.front_face)
            .line_width(1.0);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(self.enable_blending);

        let attachments = [color_blend_attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&attachments);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_module)
                .name(c"main"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(c"main"),
        ];

        let color_formats = [self.color_format];
        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_formats);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .push_next(&mut rendering_info);

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("Failed to create graphics pipeline")[0]
        };

        // Destroy shader modules
        unsafe {
            device.destroy_shader_module(vert_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        Ok((pipeline, pipeline_layout))
    }
}

/// A mesh - just geometry data (vertices and optionally indices)
/// Can be drawn by ANY compatible pipeline
pub struct Mesh<V> {
    pub vertex_buffer: VertexBuffer<V>,
    pub index_buffer: Option<IndexBuffer>,
}

impl<V> Mesh<V> {
    pub fn new(vertex_buffer: VertexBuffer<V>) -> Self {
        Self {
            vertex_buffer,
            index_buffer: None,
        }
    }

    pub fn with_indices(vertex_buffer: VertexBuffer<V>, index_buffer: IndexBuffer) -> Self {
        Self {
            vertex_buffer,
            index_buffer: Some(index_buffer),
        }
    }

    /// Draw this mesh using the current pipeline
    pub fn draw(&self, ctx: &crate::renderer::RenderContext) -> Result<()> {
        ctx.bind_vertex_buffer(self.vertex_buffer.buffer);
        
        if let Some(ref indices) = self.index_buffer {
            ctx.bind_index_buffer(indices.buffer);
            ctx.draw_indexed(indices.index_count, 1, 0, 0, 0);
        } else {
            ctx.draw(self.vertex_buffer.vertex_count, 1, 0, 0);
        }
        
        Ok(())
    }
}

fn find_memory_type(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    requirements: &vk::MemoryRequirements,
    required_properties: vk::MemoryPropertyFlags,
) -> Result<u32> {
    let mem_properties = unsafe {
        instance.get_physical_device_memory_properties(physical_device)
    };

    for i in 0..mem_properties.memory_type_count {
        if requirements.memory_type_bits & (1 << i) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties)
        {
            return Ok(i);
        }
    }

    Err(anyhow::anyhow!("Failed to find suitable memory type"))
}
