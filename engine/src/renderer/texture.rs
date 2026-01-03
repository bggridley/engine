// Create new file: engine/src/renderer/texture.rs
use anyhow::Result;
use ash::vk::{
    Image, ImageCreateInfo, SharingMode, ImageLayout, SampleCountFlags, ImageUsageFlags,
    ImageType, Extent3D, Format, MemoryPropertyFlags, ImageTiling, ImageView, ImageViewCreateInfo,
    ImageViewType, ComponentMapping, ImageSubresourceRange, ImageAspectFlags,
    CommandBuffer, CommandPool, Queue, CommandBufferAllocateInfo, CommandBufferLevel, 
    CommandBufferBeginInfo, ImageMemoryBarrier, AccessFlags, PipelineStageFlags,
    DeviceMemory,
};
use std::sync::Arc;

/// Represents a GPU texture with its image and view
pub struct Texture {
    pub image: Image,
    pub image_view: ImageView,
    pub memory: DeviceMemory,
    pub width: u32,
    pub height: u32,
    pub format: Format,
}

impl Texture {
    /// Create a texture from raw pixel data
    /// 
    /// # Arguments
    /// * `data` - Raw pixel bytes (format depends on `format` parameter)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `format` - Vulkan format (e.g., R8_UNORM for grayscale, R8G8B8A8_SRGB for color)
    /// * `device` - Vulkan device
    /// * `instance` - Vulkan instance
    /// * `physical_device` - Physical device
    /// * `queue_family_index` - Queue family for transfer operations
    pub fn from_bytes(
        data: &[u8],
        width: u32,
        height: u32,
        format: Format,
        device: &Arc<ash::Device>,
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self> {
        unsafe {
            // Create staging buffer
            let buffer_size = data.len() as u64;
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
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
            device.unmap_memory(staging_memory);
            
            // Create optimal tiled image
            let image_info = ImageCreateInfo::default()
                .image_type(ImageType::TYPE_2D)
                .format(format)
                .extent(Extent3D {
                    width,
                    height,
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

            // Transfer image data using a one-time command buffer
            Self::transition_and_copy_image(
                device,
                queue_family_index,
                image,
                staging_buffer,
                width,
                height,
            )?;

            // Clean up staging resources
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_memory, None);
            
            // Create image view
            let image_view = device.create_image_view(
                &ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(ImageViewType::TYPE_2D)
                    .format(format)
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
            )?;

            Ok(Texture {
                image,
                image_view,
                memory,
                width,
                height,
                format,
            })
        }
    }

    /// Transition image layout and copy from staging buffer
    /// This is the reusable "barrier transition" logic
    unsafe fn transition_and_copy_image(
        device: &Arc<ash::Device>,
        queue_family_index: u32,
        image: Image,
        staging_buffer: ash::vk::Buffer,
        width: u32,
        height: u32,
    ) -> Result<()> {
        // Create temporary command pool for one-time commands
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
        
        // BARRIER 1: Transition UNDEFINED → TRANSFER_DST_OPTIMAL
        // This prepares the image to receive data from the staging buffer
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
            PipelineStageFlags::TOP_OF_PIPE,  // Wait for nothing (we just created it)
            PipelineStageFlags::TRANSFER,      // Block transfer operations until transition completes
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
            .image_extent(Extent3D { width, height, depth: 1 });
        
        device.cmd_copy_buffer_to_image(
            cmd_buffer,
            staging_buffer,
            image,
            ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
        
        // BARRIER 2: Transition TRANSFER_DST_OPTIMAL → SHADER_READ_ONLY_OPTIMAL
        // This prepares the image for shader sampling (reading in fragment shaders)
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
            PipelineStageFlags::TRANSFER,          // Wait for transfer to complete
            PipelineStageFlags::FRAGMENT_SHADER,   // Block fragment shader until transition completes
            ash::vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
        
        device.end_command_buffer(cmd_buffer)?;
        
        // Submit and wait for completion
        let queue = device.get_device_queue(queue_family_index, 0);
        let command_buffers = [cmd_buffer];
        let submit_info = ash::vk::SubmitInfo::default()
            .command_buffers(&command_buffers);
        let submit_infos = [submit_info];
        device.queue_submit(queue, &submit_infos, ash::vk::Fence::null())?;
        device.queue_wait_idle(queue)?;
        
        device.destroy_command_pool(temp_pool, None);
        
        Ok(())
    }

    /// Load a texture from a PNG/JPEG file
    /// This will be useful for loading 3D textures, UI images, etc.
    pub fn from_file(
        path: &str,
        device: &Arc<ash::Device>,
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self> {
        // Load image using image crate (you'll need to add this dependency)
        let img = image::open(path)
            .map_err(|e| anyhow::anyhow!("Failed to load image '{}': {}", path, e))?;
        
        let img_rgba = img.to_rgba8();
        let (width, height) = img_rgba.dimensions();
        let data = img_rgba.as_raw();
        
        Self::from_bytes(
            data,
            width,
            height,
            Format::R8G8B8A8_SRGB,  // Standard RGBA format
            device,
            instance,
            physical_device,
            queue_family_index,
        )
    }

    pub fn destroy(&self, device: &Arc<ash::Device>) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }
}

/// Find suitable memory type for allocation
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