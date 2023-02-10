use super::constants::MAX_FRAMES_IN_FLIGHT;
use super::structures;
use super::{context::AshContext, pipeline::AshPipeline, swapchain::AshSwapchain};
use crate::adel_renderer::definitions::{UniformBufferObject, Vertex};
use anyhow::{anyhow, Result};
use ash::vk;
use image::DynamicImage;
use std::path::Path;
pub struct AshPresenter {
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    transient_command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    graphics_transient_queue: vk::Queue,

    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
}
impl AshPresenter {
    pub fn new(
        device: &ash::Device,
        context: &AshContext,
        swapchain: &AshSwapchain,
        pipeline: &AshPipeline,
    ) -> Result<Self> {
        let command_pool = AshPresenter::create_command_pool(
            &device,
            &context.queue_family,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;
        let command_buffers = AshPresenter::create_command_buffers(&device, command_pool)?;
        let graphics_transient_queue = swapchain.graphics_queue.clone();
        let (depth_image, depth_image_memory, depth_image_view) = AshPresenter::create_depth_image(
            &context,
            &device,
            swapchain.swapchain_info.swapchain_extent,
            &command_pool,
            graphics_transient_queue,
        )?;
        let framebuffers = AshPresenter::create_framebuffers(
            &device,
            pipeline.render_pass().clone(),
            &swapchain.image_views,
            depth_image_view,
            swapchain.extent(),
        )?;
        let transient_command_pool = AshPresenter::create_command_pool(
            &device,
            &context.queue_family,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?;

        Ok(Self {
            framebuffers,
            command_pool,
            transient_command_pool,
            command_buffers,
            graphics_transient_queue,
            depth_image,
            depth_image_memory,
            depth_image_view,
        })
    }
    fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        depth_image_view: vk::ImageView,
        swapchain_extent: vk::Extent2D,
    ) -> Result<Vec<vk::Framebuffer>> {
        let mut framebuffers = vec![];

        for &image_view in image_views.iter() {
            let attachments = [image_view, depth_image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1)
                .build();

            let framebuffer = unsafe { device.create_framebuffer(&framebuffer_create_info, None)? };

            framebuffers.push(framebuffer);
        }

        Ok(framebuffers)
    }

    fn create_command_pool(
        device: &ash::Device,
        queue_families: &structures::QueueFamilyIndices,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<vk::CommandPool> {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(flags)
            .queue_family_index(queue_families.graphics_family.unwrap())
            .build();

        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };
        Ok(command_pool)
    }
    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> Result<Vec<vk::CommandBuffer>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };
        Ok(command_buffers)
    }

    pub fn create_depth_image(
        context: &AshContext,
        device: &ash::Device,
        extent: vk::Extent2D,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
        let format = AshPresenter::get_depth_format(context)?;
        let (depth_image, depth_image_memory) = AshPresenter::create_image(
            context,
            device,
            extent.width,
            extent.height,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let depth_image_view = AshSwapchain::create_image_view(
            device,
            depth_image,
            format,
            vk::ImageAspectFlags::DEPTH,
            1,
        )?;

        AshPresenter::transition_image_layout(
            device,
            depth_image,
            format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            command_pool,
            submit_queue,
        )?;

        Ok((depth_image, depth_image_memory, depth_image_view))
    }
    pub fn recreate_framebuffers(
        &mut self,
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        depth_image_view: vk::ImageView,
        extent: vk::Extent2D,
    ) -> Result<()> {
        let framebuffers = AshPresenter::create_framebuffers(
            device,
            render_pass,
            image_views,
            depth_image_view,
            extent,
        )?;
        self.framebuffers = framebuffers;
        Ok(())
    }
    pub fn recreate_depth_image(
        &mut self,
        context: &AshContext,
        device: &ash::Device,
        extent: vk::Extent2D,
    ) -> Result<()> {
        let (depth_image, depth_image_memory, depth_image_view) = AshPresenter::create_depth_image(
            context,
            device,
            extent,
            &self.command_pool,
            self.graphics_transient_queue,
        )?;
        self.depth_image = depth_image;
        self.depth_image_memory = depth_image_memory;
        self.depth_image_view = depth_image_view;
        Ok(())
    }

    // Reference members of the struct functions
    pub fn framebuffers(&self) -> &Vec<vk::Framebuffer> {
        &self.framebuffers
    }

    pub fn command_buffers(&self) -> &Vec<vk::CommandBuffer> {
        &self.command_buffers
    }
    pub fn command_pool(&self) -> &vk::CommandPool {
        &self.command_pool
    }
    pub fn submit_queue(&self) -> vk::Queue {
        self.graphics_transient_queue
    }
    pub fn depth_image_view(&self) -> vk::ImageView {
        self.depth_image_view
    }
    pub fn get_depth_format(context: &AshContext) -> Result<vk::Format> {
        let candidates = &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        context.get_supported_format(
            candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    // Destroyers, called by renderer Drop method
    pub unsafe fn destroy_framebuffers(&mut self, device: &ash::Device) {
        for &framebuffer in self.framebuffers.iter() {
            device.destroy_framebuffer(framebuffer, None);
        }
    }
    pub unsafe fn free_command_buffers(&mut self, device: &ash::Device) {
        device.free_command_buffers(self.command_pool, &self.command_buffers);
    }
    pub unsafe fn destroy_command_pools(&mut self, device: &ash::Device) {
        device.destroy_command_pool(self.command_pool, None);
        device.destroy_command_pool(self.transient_command_pool, None);
    }
    pub unsafe fn destroy_depth_image(&mut self, device: &ash::Device) {
        device.destroy_image(self.depth_image, None);
        device.destroy_image_view(self.depth_image_view, None);
        device.free_memory(self.depth_image_memory, None);
    }

    /*

        Vertex/Index/Texture Buffer

    */
    pub fn create_buffer(
        context: &AshContext,
        device: &ash::Device,
        device_size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(device_size)
            .usage(usage)
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
            properties,
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
        Ok((buffer, buffer_memory))
    }

    fn copy_buffer(
        device: &ash::Device,
        src_buffer: &vk::Buffer,
        dst_buffer: &vk::Buffer,
        size: vk::DeviceSize,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<()> {
        let command_buffer = AshPresenter::begin_single_time_commands(device, command_pool)?;

        let copy_region = [vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(size)
            .build()];

        unsafe {
            device.cmd_copy_buffer(command_buffer, *src_buffer, *dst_buffer, &copy_region);
        };
        AshPresenter::end_single_time_commands(device, command_buffer, command_pool, submit_queue)?;
        Ok(())
    }
    pub fn create_vertex_buffer(
        &self,
        context: &AshContext,
        device: &ash::Device,
        vertices: &Vec<Vertex>,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        //let buffer_size = std::mem::size_of_val(&triangle.verticies) as vk::DeviceSize;
        let buffer_size = (vertices.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize;
        //log::info!("JAKOB buffer 1 {:?} buffer 2 {:?}", buffer_size, buffer_size2);
        let (staging_buffer, staging_buffer_memory) = AshPresenter::create_buffer(
            context,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            device.unmap_memory(staging_buffer_memory);
        }
        let (vertex_buffer, vertex_buffer_memory) = AshPresenter::create_buffer(
            context,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        AshPresenter::copy_buffer(
            device,
            &staging_buffer,
            &vertex_buffer,
            buffer_size,
            command_pool,
            submit_queue,
        )?;

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        Ok((vertex_buffer, vertex_buffer_memory))
    }

    pub fn create_index_buffer(
        &self,
        context: &AshContext,
        device: &ash::Device,
        indicies: &Vec<u32>,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (indicies.len() * std::mem::size_of::<u32>()) as vk::DeviceSize;
        //log::info!("JAKOB buffer 1 {:?} buffer 2 {:?}", buffer_size, buffer_size2);
        let (staging_buffer, staging_buffer_memory) = AshPresenter::create_buffer(
            context,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut u32;

            data_ptr.copy_from_nonoverlapping(indicies.as_ptr(), indicies.len());

            device.unmap_memory(staging_buffer_memory);
        }
        let (index_buffer, index_buffer_memory) = AshPresenter::create_buffer(
            context,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        AshPresenter::copy_buffer(
            device,
            &staging_buffer,
            &index_buffer,
            buffer_size,
            command_pool,
            submit_queue,
        )?;

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        Ok((index_buffer, index_buffer_memory))
    }
    pub fn create_uniform_buffers(
        context: &AshContext,
        device: &ash::Device,
    ) -> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
        let buffer_size: vk::DeviceSize =
            std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;
        let mut uniform_buffers: Vec<vk::Buffer> = Vec::new();
        let mut uniform_buffers_memory: Vec<vk::DeviceMemory> = Vec::new();
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (uniform_buffer, uniform_buffer_memory) = AshPresenter::create_buffer(
                context,
                device,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }

        Ok((uniform_buffers, uniform_buffers_memory))
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
        let (staging_buffer, staging_buffer_memory) = AshPresenter::create_buffer(
            context,
            device,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        let image_data = image_data.into_raw();
        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer_memory,
                0,
                image_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_size as usize);

            device.unmap_memory(staging_buffer_memory);
        }
        let (texture_image, texture_image_memory) = AshPresenter::create_image(
            context,
            device,
            image_width,
            image_height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        AshPresenter::transition_image_layout(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            command_pool,
            submit_queue,
        )?;
        AshPresenter::copy_buffer_to_image(
            device,
            staging_buffer,
            texture_image,
            image_width,
            image_height,
            command_pool,
            submit_queue,
        )?;
        AshPresenter::transition_image_layout(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            command_pool,
            submit_queue,
        )?;
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }
        Ok((texture_image, texture_image_memory))
    }

    pub fn update_uniform_buffer(
        &self,
        device: &ash::Device,
        current_image: usize,
        uniform_buffers_memory: &Vec<vk::DeviceMemory>,
        proj: nalgebra::Matrix4<f32>,
    ) -> Result<()> {
        let ubos = [UniformBufferObject {
            model: nalgebra::Matrix4::<f32>::identity(),
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
    }
    pub fn update_uniform_buffer_new(
        device: &ash::Device,
        uniform_buffers_memory: &Vec<vk::DeviceMemory>,
        current_image: usize,
        model: nalgebra::Matrix4<f32>,
        proj: nalgebra::Matrix4<f32>,
        view: nalgebra::Matrix4<f32>,
    ) -> Result<()> {
        let ubos = [UniformBufferObject { model, view, proj }];
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
    }
    pub fn create_texture_image_bak(
        context: &AshContext,
        device: &ash::Device,
        submit_queue: vk::Queue,
        image_path: &Path,
        command_pool: &vk::CommandPool,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        let image_object: DynamicImage = image::open(image_path).unwrap();
        //image_object = image_object.flipv();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        // Size is u8 - per color size, 4 - rgba, width*height - area
        let image_size =
            (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        // This crushes 16/32 bit pixel definition to 8 bit
        let image_data = image_object.into_rgba8().into_raw();

        let (staging_buffer, staging_buffer_memory) = AshPresenter::create_buffer(
            context,
            device,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        unsafe {
            let data_ptr = device.map_memory(
                staging_buffer_memory,
                0,
                image_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut u8;

            data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_size as usize);

            device.unmap_memory(staging_buffer_memory);
        }
        let (texture_image, texture_image_memory) = AshPresenter::create_image(
            context,
            device,
            image_width,
            image_height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        AshPresenter::transition_image_layout(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            command_pool,
            submit_queue,
        )?;
        AshPresenter::copy_buffer_to_image(
            device,
            staging_buffer,
            texture_image,
            image_width,
            image_height,
            command_pool,
            submit_queue,
        )?;
        AshPresenter::transition_image_layout(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            command_pool,
            submit_queue,
        )?;
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }
        Ok((texture_image, texture_image_memory))
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
    fn create_image(
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
            device
                .queue_wait_idle(submit_queue)
                .expect("Failed to wait idle");
            device.free_command_buffers(*command_pool, command_buffers);
        }
        Ok(())
    }
    fn transition_image_layout(
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

        let command_buffer = AshPresenter::begin_single_time_commands(device, command_pool)?;
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
        AshPresenter::end_single_time_commands(device, command_buffer, command_pool, submit_queue)?;
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
        let command_buffer = AshPresenter::begin_single_time_commands(device, command_pool)?;
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

        AshPresenter::end_single_time_commands(device, command_buffer, command_pool, submit_queue)?;
        Ok(())
    }

    pub unsafe fn destroy_all(&mut self, device: &ash::Device) {
        self.destroy_depth_image(device);
        // Framebuffers, Commandbuffers, and CommandPool need cleanup
        // Framebuffers need to be separated as they are removed when recreating swapchain
        self.destroy_framebuffers(device);
        self.free_command_buffers(device);
        self.destroy_command_pools(device);
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
