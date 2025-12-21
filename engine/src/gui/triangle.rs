use anyhow::Result;
use ash::vk::{self, ShaderStageFlags};
use std::sync::Arc;

use crate::renderer::{ShaderId, ShaderManager};
use crate::gui::GUIComponent;
use glam::Vec2;

#[repr(C)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

pub struct TriangleComponent {
    device: Arc<ash::Device>,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pos: glam::Vec2,
}


impl GUIComponent for TriangleComponent {
    fn render(&self, ctx: &crate::renderer::RenderContext) -> Result<()> {
        ctx.bind_pipeline(self.pipeline);
        ctx.bind_vertex_buffer(self.vertex_buffer);
        ctx.draw(3, 1, 0, 0);
        Ok(())
    }

    fn set_pos(&mut self, _x: f32, _y: f32) {
        // Triangle position is fixed; no-op
        self.pos.x = _x;
        self.pos.y = _y;
    }
}

impl TriangleComponent {
    pub fn new(context: &Arc<crate::renderer::VulkanContext>) -> Result<Self> {
        // Compile shaders from files
        let pos = Vec2::new(0.0, 0.0);
        let shader_manager = ShaderManager::new()?;
        shader_manager.compile_all_shaders()?;

        let vert_shader_code = ShaderId::TriangleVertex.load_shader_bytes()?;
        let frag_shader_code = ShaderId::TriangleFrag.load_shader_bytes()?;

        // Create shader modules
        let vert_module = unsafe {
            context.device.create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&vert_shader_code),
                None,
            )?
        };

        let frag_module = unsafe {
            context.device.create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&frag_shader_code),
                None,
            )?
        };

        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<[f32; 4]>() as u32 * 4);

        // Create pipeline layout
        let pipeline_layout = unsafe {
            context.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&[push_constant_range]),
                None,
            )?
        };

        // Vertex input state
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)];

        let vertex_input_attribute_descriptions = [
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

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_input_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let attachments = [color_blend_attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&attachments);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

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

        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&[vk::Format::B8G8R8A8_SRGB]);

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
            context
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("Failed to create graphics pipeline")[0]
        };

        // Destroy shader modules
        unsafe {
            context.device.destroy_shader_module(vert_module, None);
            context.device.destroy_shader_module(frag_module, None);
        }

        // Create vertex buffer with triangle data
        let vertices = [
            Vertex {
                position: [0.0, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
        ];

        let buffer_info = vk::BufferCreateInfo::default()
            .size(std::mem::size_of_val(&vertices) as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let vertex_buffer = unsafe { context.device.create_buffer(&buffer_info, None)? };

        let mem_requirements =
            unsafe { context.device.get_buffer_memory_requirements(vertex_buffer) };

        // Find suitable memory type
        let mem_type_index = find_memory_type(
            &context.instance,
            context.physical_device,
            &mem_requirements,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let vertex_buffer_memory = unsafe { context.device.allocate_memory(&alloc_info, None)? };

        unsafe {
            context
                .device
                .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)?;
        }

        // Copy vertex data to buffer
        unsafe {
            let data_ptr = context.device.map_memory(
                vertex_buffer_memory,
                0,
                mem_requirements.size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const u8,
                data_ptr as *mut u8,
                std::mem::size_of_val(&vertices),
            );
            context.device.unmap_memory(vertex_buffer_memory);
        }

        Ok(TriangleComponent {
            device: context.device.clone(),
            pipeline,
            pipeline_layout,
            vertex_buffer,
            vertex_buffer_memory,
            pos,
        })
    }

    pub fn draw(&self, device: &Arc<ash::Device>, cmd_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            device.cmd_bind_vertex_buffers(cmd_buffer, 0, &[self.vertex_buffer], &[0]);
            device.cmd_draw(cmd_buffer, 3, 1, 0, 0);
        }
    }
}

impl Drop for TriangleComponent {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

fn find_memory_type(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    mem_requirements: &vk::MemoryRequirements,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32> {
    let mem_props = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    for i in 0..mem_props.memory_type_count {
        if (mem_requirements.memory_type_bits & (1 << i)) != 0
            && (mem_props.memory_types[i as usize].property_flags & properties) == properties
        {
            return Ok(i);
        }
    }

    anyhow::bail!("No suitable memory type found")
}
