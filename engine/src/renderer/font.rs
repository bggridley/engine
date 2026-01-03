use anyhow::Result;
use ash::vk::{
    Image, ImageCreateInfo, SharingMode, ImageLayout, SampleCountFlags, ImageUsageFlags,
    ImageType, Extent3D, Format, MemoryPropertyFlags, ImageTiling,
};
use glam::Vec2;
use rusttype::{point, Font, Scale};
use std::{collections::HashMap, sync::Arc};
pub struct FontAtlas {
    pub texture: Image,
    pub glyph_map: HashMap<char, GlyphMetrics>,
}

#[derive(Clone, Copy, Debug)]
struct GlyphMetrics {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub advance_width: f32,
    pub bearing_y: f32,
}

const CHARS_TO_RASTERIZE: &str =
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

impl FontAtlas {
    pub fn load(
        path: &str,
        device: &Arc<ash::Device>,
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
    ) -> Result<Self> {
        let font_data = std::fs::read(path)?;
        let font = Font::try_from_vec(font_data).unwrap();

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
            .map(|(g, ch)| {
                let bb = g.pixel_bounding_box().unwrap();
                (
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
                    },
                )
            })
            .collect();

        // TODO: Create Vulkan texture from pixels
        let texture = unsafe {
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
                .tiling(ImageTiling::LINEAR)
                .usage(ImageUsageFlags::SAMPLED)
                .sharing_mode(SharingMode::EXCLUSIVE)
                .initial_layout(ImageLayout::UNDEFINED);

            let image = device.create_image(&image_info, None)?;
            let mem_req = device.get_image_memory_requirements(image);

            let mem_type = find_memory_type(
                instance,
                physical_device,
                &mem_req,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            )?;

            let alloc_info = ash::vk::MemoryAllocateInfo::default()
                .allocation_size(mem_req.size)
                .memory_type_index(mem_type);

            let memory = device.allocate_memory(&alloc_info, None)?;
            device.bind_image_memory(image, memory, 0)?;

            let ptr = device.map_memory(memory, 0, mem_req.size, ash::vk::MemoryMapFlags::empty())?;
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), ptr as *mut u8, pixels.len());
            device.unmap_memory(memory);

            image
        };


        Ok(FontAtlas { texture, glyph_map })
    }

    pub fn get_text_width(&self, text: &str) -> f32 {
        text.chars()
            .filter_map(|c| self.glyph_map.get(&c))
            .map(|metrics| metrics.advance_width)
            .sum()
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
