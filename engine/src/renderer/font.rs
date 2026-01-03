use anyhow::Result;
use ash::vk::{
    Image, ImageCreateInfo, SharingMode, ImageLayout, SampleCountFlags, ImageUsageFlags,
    ImageType, Extent3D, Format, MemoryPropertyFlags, ImageTiling, ImageView, ImageViewCreateInfo,
    ImageViewType, ComponentMapping, ImageSubresourceRange, ImageAspectFlags,
    CommandBuffer, CommandPool, Queue, CommandBufferAllocateInfo, CommandBufferLevel, 
    CommandBufferBeginInfo, ImageMemoryBarrier, AccessFlags, PipelineStageFlags,
};
use glam::Vec2;
use rusttype::{point, Font, Scale};
use std::{collections::HashMap, sync::Arc};
pub struct FontAtlas {
    pub texture: Image,
    pub texture_view: ImageView,
    pub glyph_map: HashMap<char, GlyphMetrics>,
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphMetrics {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub advance_width: f32,
    pub bearing_y: f32,
    pub width: f32,   // Pixel width in the rasterized texture
    pub height: f32,  // Pixel height in the rasterized texture
}

const CHARS_TO_RASTERIZE: &str =
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

impl FontAtlas {
    pub fn load(
        path: &str,
        device: &Arc<ash::Device>,
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self> {
        let font_data = std::fs::read(path)
            .map_err(|e| anyhow::anyhow!("Failed to load font file '{}': {}", path, e))?;
        let font = Font::try_from_vec(font_data)
            .ok_or_else(|| anyhow::anyhow!("Invalid font file format"))?;

        let height: f32 = 128.0;
        let scale = Scale {
            x: height,
            y: height,
        };

        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        let glyphs: Vec<_> = font.layout(CHARS_TO_RASTERIZE, scale, offset).collect();

        // Calculate texture width from rightmost glyph
        let texture_width = glyphs
            .iter()
            .map(|g| {
                let bb = g.pixel_bounding_box().unwrap_or_default();
                (bb.max.x as u32)
                    .max(g.position().x as u32 + g.unpositioned().h_metrics().advance_width as u32)
            })
            .max()
            .unwrap_or(512) as usize;

        let texture_height = height.ceil() as usize;

        // Create pixel buffer
        let mut pixels = vec![0u8; texture_width * texture_height];

        // Draw glyphs into buffer
        for glyph in &glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let px = (bb.min.x + x as i32) as usize;
                    let py = (bb.min.y + y as i32) as usize;
                    if px < texture_width && py < texture_height {
                        pixels[py * texture_width + px] = (v * 255.0) as u8;
                    }
                });
            }
        }

        // Build glyph map
        let glyph_map = glyphs
            .iter()
            .zip(CHARS_TO_RASTERIZE.chars())
            .filter_map(|(g, ch)| {
                // Skip glyphs without bounding boxes (e.g., space)
                let bb = g.pixel_bounding_box()?;
                let width = (bb.max.x - bb.min.x) as f32;
                let height = (bb.max.y - bb.min.y) as f32;
                Some((
                    ch,
                    GlyphMetrics {
                        uv_min: Vec2::new(
                            bb.min.x as f32 / texture_width as f32,
                            bb.min.y as f32 / texture_height as f32,
                        ),
                        uv_max: Vec2::new(
                            bb.max.x as f32 / texture_width as f32,
                            bb.max.y as f32 / texture_height as f32,
                        ),
                        advance_width: g.unpositioned().h_metrics().advance_width,
                        bearing_y: bb.max.y as f32,
                        width,
                        height,
                    },
                ))
            })
            .collect();

        // Create Vulkan texture from pixels
        let texture = unsafe {
            // Create staging buffer
            let buffer_size = (texture_width * texture_height) as u64;
            let staging_buffer_info = ash::vk::BufferCreateInfo::default()
                .size(buffer_size)
                .usage(ash::vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(SharingMode::EXCLUSIVE);
            
            let staging_buffer = device.create_buffer(&staging_buffer_info, None)?;
            let staging_mem_req = device.get_buffer_memory_requirements(staging_buffer);
            
            let staging_mem_type = find_memory_type(
                instance,
                physical_device,
                &staging_mem_req,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            )?;
            
            let staging_alloc_info = ash::vk::MemoryAllocateInfo::default()
                .allocation_size(staging_mem_req.size)
                .memory_type_index(staging_mem_type);
            
            let staging_memory = device.allocate_memory(&staging_alloc_info, None)?;
            device.bind_buffer_memory(staging_buffer, staging_memory, 0)?;
            
            // Copy pixel data to staging buffer
            let ptr = device.map_memory(staging_memory, 0, buffer_size, ash::vk::MemoryMapFlags::empty())?;
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), ptr as *mut u8, pixels.len());
            device.unmap_memory(staging_memory);
            
            // Create optimal tiled image
            let image_info = ImageCreateInfo::default()
                .image_type(ImageType::TYPE_2D)
                .format(Format::R8_UNORM)
                .extent(Extent3D {
                    width: texture_width as u32,
                    height: texture_height as u32,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(SampleCountFlags::TYPE_1)
                .tiling(ImageTiling::OPTIMAL)
                .usage(ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED)
                .sharing_mode(SharingMode::EXCLUSIVE)
                .initial_layout(ImageLayout::UNDEFINED);

            let image = device.create_image(&image_info, None)?;
            let mem_req = device.get_image_memory_requirements(image);

            let mem_type = find_memory_type(
                instance,
                physical_device,
                &mem_req,
                MemoryPropertyFlags::DEVICE_LOCAL,
            )?;

            let alloc_info = ash::vk::MemoryAllocateInfo::default()
                .allocation_size(mem_req.size)
                .memory_type_index(mem_type);

            let memory = device.allocate_memory(&alloc_info, None)?;
            device.bind_image_memory(image, memory, 0)?;

            // Transition image layout from UNDEFINED to SHADER_READ_ONLY_OPTIMAL
            // Create temporary command pool and queue for one-time command
            let pool_create_info = ash::vk::CommandPoolCreateInfo::default()
                .flags(ash::vk::CommandPoolCreateFlags::TRANSIENT)
                .queue_family_index(queue_family_index);
            
            let temp_pool = device.create_command_pool(&pool_create_info, None)?;
            
            let alloc_info = CommandBufferAllocateInfo::default()
                .command_pool(temp_pool)
                .level(CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            
            let cmd_buffers = device.allocate_command_buffers(&alloc_info)?;
            let cmd_buffer = cmd_buffers[0];
            
            let begin_info = CommandBufferBeginInfo::default()
                .flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(cmd_buffer, &begin_info)?;
            
            // Transition to TRANSFER_DST_OPTIMAL for copying
            let barrier = ImageMemoryBarrier::default()
                .old_layout(ImageLayout::UNDEFINED)
                .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(
                    ImageSubresourceRange::default()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                )
                .src_access_mask(AccessFlags::empty())
                .dst_access_mask(AccessFlags::TRANSFER_WRITE);
            
            device.cmd_pipeline_barrier(
                cmd_buffer,
                PipelineStageFlags::TOP_OF_PIPE,
                PipelineStageFlags::TRANSFER,
                ash::vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
            
            // Copy buffer to image
            let region = ash::vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(
                    ash::vk::ImageSubresourceLayers::default()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1)
                )
                .image_offset(ash::vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(Extent3D {
                    width: texture_width as u32,
                    height: texture_height as u32,
                    depth: 1,
                });
            
            device.cmd_copy_buffer_to_image(
                cmd_buffer,
                staging_buffer,
                image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
            
            // Transition to SHADER_READ_ONLY_OPTIMAL for sampling
            let barrier = ImageMemoryBarrier::default()
                .old_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(
                    ImageSubresourceRange::default()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                )
                .src_access_mask(AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(AccessFlags::SHADER_READ);
            
            device.cmd_pipeline_barrier(
                cmd_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::FRAGMENT_SHADER,
                ash::vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
            
            device.end_command_buffer(cmd_buffer)?;
            
            let queue = device.get_device_queue(queue_family_index, 0);
            let command_buffers = [cmd_buffer];
            let submit_info = ash::vk::SubmitInfo::default()
                .command_buffers(&command_buffers);
            let submit_infos = [submit_info];
            device.queue_submit(queue, &submit_infos, ash::vk::Fence::null())?;
            device.queue_wait_idle(queue)?;
            
            device.destroy_command_pool(temp_pool, None);
            
            // Clean up staging resources
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_memory, None);
            
            image
        };

        // Create image view for the texture
        let texture_view = unsafe {
            device.create_image_view(
                &ImageViewCreateInfo::default()
                    .image(texture)
                    .view_type(ImageViewType::TYPE_2D)
                    .format(Format::R8_UNORM)
                    .components(ComponentMapping {
                        r: ash::vk::ComponentSwizzle::IDENTITY,
                        g: ash::vk::ComponentSwizzle::IDENTITY,
                        b: ash::vk::ComponentSwizzle::IDENTITY,
                        a: ash::vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(ImageSubresourceRange {
                        aspect_mask: ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    }),
                None,
            )?
        };

        Ok(FontAtlas { texture, texture_view, glyph_map })
    }

    pub fn get_text_width(&self, text: &str) -> f32 {
        text.chars()
            .filter_map(|c| self.glyph_map.get(&c))
            .map(|metrics| metrics.advance_width)
            .sum()
    }

    pub fn get_glyph(&self, ch: char) -> Option<&GlyphMetrics> {
        self.glyph_map.get(&ch)
    }
}

fn find_memory_type(
    instance: &ash::Instance,
    physical_device: ash::vk::PhysicalDevice,
    mem_req: &ash::vk::MemoryRequirements,
    properties: MemoryPropertyFlags,
) -> Result<u32> {
    let mem_props = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    for i in 0..mem_props.memory_type_count {
        if (mem_req.memory_type_bits & (1 << i)) != 0
            && (mem_props.memory_types[i as usize].property_flags & properties) == properties
        {
            return Ok(i);
        }
    }
    Err(anyhow::anyhow!("No suitable memory type found"))
}
