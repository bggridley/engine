use anyhow::Result;
use ash::vk;
use std::sync::Arc;

/// Find suitable memory type for allocation
/// Used by buffers, images, and any Vulkan memory allocation
pub fn find_memory_type(
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
    
    Err(anyhow::anyhow!("No suitable memory type found"))
}

/// Create a host-visible buffer and copy data into it
/// Generic function for vertex buffers, index buffers, uniform buffers, etc.
/// 
/// # Arguments
/// * `data` - The data to copy into the buffer
/// * `usage` - Buffer usage flags (e.g., VERTEX_BUFFER, INDEX_BUFFER)
pub fn create_buffer_with_data<T>(
    device: &Arc<ash::Device>,
    physical_device: vk::PhysicalDevice,
    instance: &ash::Instance,
    data: &[T],
    usage: vk::BufferUsageFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_size = std::mem::size_of_val(data) as vk::DeviceSize;
    
    let buffer_info = vk::BufferCreateInfo::default()
        .size(buffer_size)
        .usage(usage)
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
        
        // Copy data to buffer
        let data_ptr = device.map_memory(
            memory,
            0,
            mem_requirements.size,
            vk::MemoryMapFlags::empty(),
        )?;
        std::ptr::copy_nonoverlapping(
            data.as_ptr() as *const u8,
            data_ptr as *mut u8,
            std::mem::size_of_val(data),
        );
        device.unmap_memory(memory);
    }

    Ok((buffer, memory))
}
