use ash::vk;

use super::command_buffers::AshCommandBuffers;
use super::constants::MAX_FRAMES_IN_FLIGHT;
use super::{context::AshContext, swapchain::AshSwapchain};
use crate::adel_renderer::definitions::{PointLightComponent, UniformBufferObject, Vertex};
use anyhow::{anyhow, Result};
use image::DynamicImage;
use nalgebra::Vector4;
use std::path::Path;

pub struct AshBuffer {
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,

    buffer_size: vk::DeviceSize,
    instance_count: u64,
    instance_size: vk::DeviceSize,
    alignment_size: vk::DeviceSize,
    buffer_usage_flags: vk::BufferUsageFlags,
    memory_property_flags: vk::MemoryPropertyFlags,
}

/*

    Vertex/Index/Texture Buffer

*/
impl AshBuffer {
    pub fn create_buffer(
        context: &AshContext,
        device: &ash::Device,
        instance_size: vk::DeviceSize,
        instance_count: vk::DeviceSize,
        buffer_usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let min_offset_alignment: vk::DeviceSize = 1;

        let alignment_size = get_alignment(instance_size, min_offset_alignment);
        let buffer_size = alignment_size * instance_count;
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(buffer_usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            //.queue_family_indices(0)
            .build();

        let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let mem_properties = unsafe {
            context
                .instance
                .get_physical_device_memory_properties(context.physical_device)
        };
        let memory_type = find_memory_type(
            mem_requirements.memory_type_bits,
            memory_property_flags,
            mem_properties,
        )?;

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type)
            .build();

        let buffer_memory = unsafe { device.allocate_memory(&allocate_info, None)? };

        unsafe {
            device.bind_buffer_memory(buffer, buffer_memory, 0)?;
        }
        Ok(Self {
            buffer,
            buffer_memory,
            buffer_size,
            instance_count,
            instance_size,
            alignment_size,
            buffer_usage_flags,
            memory_property_flags,
        })
    }

    fn copy_buffer(
        device: &ash::Device,
        src_buffer: &vk::Buffer,
        dst_buffer: &vk::Buffer,
        size: vk::DeviceSize,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<()> {
        let command_buffer = AshBuffer::begin_single_time_commands(device, command_pool)?;

        let copy_region = [vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(size)
            .build()];

        unsafe {
            device.cmd_copy_buffer(command_buffer, *src_buffer, *dst_buffer, &copy_region);
        };
        AshBuffer::end_single_time_commands(device, command_buffer, command_pool, submit_queue)?;
        Ok(())
    }
    pub fn flush_buffer(&self, device: &ash::Device) -> Result<()> {
        // TODO: Pass in vk::WHOLE_SIZE or buffer size?
        let mapped_memory_range = [vk::MappedMemoryRange::builder()
            .memory(self.memory())
            .size(vk::WHOLE_SIZE)
            .offset(0)
            .build()];
        unsafe {
            device.flush_mapped_memory_ranges(&mapped_memory_range)?;
        }
        Ok(())
    }
    pub fn create_vertex_buffer(
        context: &AshContext,
        device: &ash::Device,
        vertices: &Vec<Vertex>,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<Self> {
        let vertex_size = std::mem::size_of::<Vertex>() as vk::DeviceSize;
        let vertex_count = vertices.len() as vk::DeviceSize;
        let staging_buffer = AshBuffer::create_buffer(
            context,
            device,
            vertex_size,
            vertex_count,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer.memory(),
                0,
                staging_buffer.buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            device.unmap_memory(staging_buffer.memory());
        }
        let vertex_buffer = AshBuffer::create_buffer(
            context,
            device,
            vertex_size,
            vertex_count,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        AshBuffer::copy_buffer(
            device,
            &staging_buffer.buffer(),
            &vertex_buffer.buffer(),
            vertex_buffer.buffer_size,
            command_pool,
            submit_queue,
        )?;

        unsafe {
            device.destroy_buffer(staging_buffer.buffer(), None);
            device.free_memory(staging_buffer.memory(), None);
        }

        Ok(vertex_buffer)
    }

    pub fn create_index_buffer(
        context: &AshContext,
        device: &ash::Device,
        indicies: &Vec<u32>,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<Self> {
        let index_size = std::mem::size_of::<u32>() as vk::DeviceSize;
        let index_count = indicies.len() as vk::DeviceSize;
        //log::info!("JAKOB buffer 1 {:?} buffer 2 {:?}", buffer_size, buffer_size2);
        let staging_buffer = AshBuffer::create_buffer(
            context,
            device,
            index_size,
            index_count,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer.memory(),
                0,
                staging_buffer.buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut u32;

            data_ptr.copy_from_nonoverlapping(indicies.as_ptr(), indicies.len());

            device.unmap_memory(staging_buffer.memory());
        }
        let index_buffer = AshBuffer::create_buffer(
            context,
            device,
            index_size,
            index_count,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        AshBuffer::copy_buffer(
            device,
            &staging_buffer.buffer(),
            &index_buffer.buffer(),
            index_buffer.buffer_size,
            command_pool,
            submit_queue,
        )?;

        unsafe {
            device.destroy_buffer(staging_buffer.buffer(), None);
            device.free_memory(staging_buffer.memory(), None);
        }

        Ok(index_buffer)
    }
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }
    pub fn buffer_ref(&self) -> &vk::Buffer {
        &self.buffer
    }
    pub fn memory(&self) -> vk::DeviceMemory {
        self.buffer_memory
    }
    pub fn memory_ref(&self) -> &vk::DeviceMemory {
        &self.buffer_memory
    }
    /*
        Descriptor Set Buffers
    */
    pub fn create_uniform_buffers(
        context: &AshContext,
        device: &ash::Device,
    ) -> Result<(Vec<Self>, Vec<*mut UniformBufferObject>)> {
        let instance_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;
        let mut uniform_buffers: Vec<Self> = Vec::new();
        let mut uniform_buffers_mapped: Vec<*mut UniformBufferObject> = Vec::new();
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let uniform_buffer = AshBuffer::create_buffer(
                context,
                device,
                instance_size,
                1,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // TODO: Reorganize this Function
            let data_ptr = unsafe {
                device.map_memory(
                    uniform_buffer.memory(),
                    0,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )?
            } as *mut UniformBufferObject;
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_mapped.push(data_ptr);
        }

        Ok((uniform_buffers, uniform_buffers_mapped))
    }

    pub fn create_texture_image(
        context: &AshContext,
        device: &ash::Device,
        image_width: u32,
        image_height: u32,
        image_size: vk::DeviceSize,
        image_data: image::RgbaImage,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        let staging_buffer = AshBuffer::create_buffer(
            context,
            device,
            image_size,
            1,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        let image_data = image_data.into_raw();
        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer.memory(),
                0,
                image_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_size as usize);

            device.unmap_memory(staging_buffer.memory());
        }
        let (texture_image, texture_image_memory) = AshBuffer::create_image(
            context,
            device,
            image_width,
            image_height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        AshBuffer::transition_image_layout(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            command_pool,
            submit_queue,
        )?;
        AshBuffer::copy_buffer_to_image(
            device,
            staging_buffer.buffer(),
            texture_image,
            image_width,
            image_height,
            command_pool,
            submit_queue,
        )?;
        AshBuffer::transition_image_layout(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            command_pool,
            submit_queue,
        )?;
        unsafe {
            device.destroy_buffer(staging_buffer.buffer(), None);
            device.free_memory(staging_buffer.memory(), None);
        }
        Ok((texture_image, texture_image_memory))
    }

    /*pub fn update_uniform_buffer(
        &self,
        device: &ash::Device,
        current_image: usize,
        uniform_buffers_memory: &Vec<vk::DeviceMemory>,
        proj: nalgebra::Matrix4<f32>,
    ) -> Result<()> {
        let ubos = [UniformBufferObject {
            view: nalgebra::Matrix4::<f32>::identity(),
            proj,
        }];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = device.map_memory(
                uniform_buffers_memory[current_image],
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            device.unmap_memory(uniform_buffers_memory[current_image]);
        }
        Ok(())
    }*/
    /*pub fn update_uniform_buffer_mvp(
        device: &ash::Device,
        uniform_buffers_memory: &Vec<vk::DeviceMemory>,
        current_image: usize,
        view: nalgebra::Matrix4<f32>,
        proj: nalgebra::Matrix4<f32>,
    ) -> Result<()> {
        let ubos = [UniformBufferObject { view, proj }];
        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = device.map_memory(
                uniform_buffers_memory[current_image],
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            device.unmap_memory(uniform_buffers_memory[current_image]);
        }
        Ok(())
    }*/
    pub fn update_global_uniform_buffer(
        device: &ash::Device,
        uniform_buffer: &AshBuffer,
        global_ubo: UniformBufferObject,
        uniform_buffer_mapped: *mut UniformBufferObject,
    ) -> Result<()> {
        let ubos = [global_ubo];
        unsafe {
            uniform_buffer_mapped.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            uniform_buffer.flush_buffer(device)?;
        }
        Ok(())
    }
    pub fn create_texture_image_view(
        device: &ash::Device,
        image: vk::Image,
    ) -> Result<vk::ImageView> {
        AshSwapchain::create_image_view(
            device,
            image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
            1,
        )
    }
    pub fn create_image(
        context: &AshContext,
        device: &ash::Device,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let texture_image = unsafe {
            device
                .create_image(&info, None)
                .expect("Failed to create image")
        };

        let tex_mem_requirements = unsafe { device.get_image_memory_requirements(texture_image) };
        let tex_mem_properties = unsafe {
            context
                .instance
                .get_physical_device_memory_properties(context.physical_device)
        };
        let tex_memory_type = find_memory_type(
            tex_mem_requirements.memory_type_bits,
            properties,
            tex_mem_properties,
        )?;
        let info = vk::MemoryAllocateInfo::builder()
            .allocation_size(tex_mem_requirements.size)
            .memory_type_index(tex_memory_type)
            .build();

        let texture_image_memory = unsafe { device.allocate_memory(&info, None)? };
        unsafe { device.bind_image_memory(texture_image, texture_image_memory, 0)? }

        Ok((texture_image, texture_image_memory))
    }

    pub fn transition_image_layout(
        device: &ash::Device,
        image: vk::Image,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        // This function can be altered when this is ran at startup rather than initialization
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<()> {
        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            match format {
                vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => {
                    vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
                }
                _ => vk::ImageAspectFlags::DEPTH,
            }
        } else {
            vk::ImageAspectFlags::COLOR
        };
        let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
            match (old_layout, new_layout) {
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => {
                    (
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                    )
                }
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                ),
                (
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                ) => (
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                ),
                _ => return Err(anyhow!("Unsupported image layout transition")),
            };

        let command_buffer = AshBuffer::begin_single_time_commands(device, command_pool)?;
        let subresource = vk::ImageSubresourceRange::builder()
            //.aspect_mask(vk::ImageAspectFlags::COLOR)
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();
        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .build();
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );
        }
        AshBuffer::end_single_time_commands(device, command_buffer, command_pool, submit_queue)?;
        Ok(())
    }
    pub fn create_texture_sample(device: &ash::Device) -> Result<vk::Sampler> {
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .build();

        let sampler = unsafe { device.create_sampler(&sampler_info, None)? };
        Ok(sampler)
    }
    fn copy_buffer_to_image(
        device: &ash::Device,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<()> {
        let command_buffer = AshBuffer::begin_single_time_commands(device, command_pool)?;
        let subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(subresource)
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(
                vk::Extent3D::builder()
                    .width(width)
                    .height(height)
                    .depth(1)
                    .build(),
            )
            .build();

        unsafe {
            device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        AshBuffer::end_single_time_commands(device, command_buffer, command_pool, submit_queue)?;
        Ok(())
    }
    /*
        Buffer Utility functions
    */
    fn begin_single_time_commands(
        device: &ash::Device,
        command_pool: &vk::CommandPool,
    ) -> Result<vk::CommandBuffer> {
        let command_buffer_alloc = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(*command_pool)
            .command_buffer_count(1)
            .build();
        let command_buffers = unsafe { device.allocate_command_buffers(&command_buffer_alloc)? };
        assert_eq!(1, command_buffers.len());
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        unsafe { device.begin_command_buffer(command_buffers[0], &begin_info)? };
        Ok(command_buffers[0])
    }
    fn end_single_time_commands(
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<()> {
        unsafe {
            device.end_command_buffer(command_buffer)?;
        }

        let command_buffers = &[command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(command_buffers)
            .build();
        unsafe {
            // Now that the command buffer has the copy command loaded, execute it
            device.queue_submit(submit_queue, &[submit_info], vk::Fence::null())?;
            device.queue_wait_idle(submit_queue)?;
            device.free_command_buffers(*command_pool, command_buffers);
        }
        Ok(())
    }
}

pub fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
) -> Result<u32> {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        //if (type_filter & (1 << i)) > 0 && (memory_type.property_flags & required_properties) == required_properties {
        //    return i as u32
        // }

        // same implementation
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties)
        {
            return Ok(i as u32);
        }
    }

    Err(anyhow!("Failed to find suitable memory type!"))
}
// TODO: Better understand alignments
pub fn get_alignment(
    instance_size: vk::DeviceSize,
    min_offset_alignment: vk::DeviceSize,
) -> vk::DeviceSize {
    if (min_offset_alignment > 0) {
        return (instance_size + min_offset_alignment - 1) & !(min_offset_alignment - 1);
    }
    return instance_size;
}
