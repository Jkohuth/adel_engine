use ash::vk;
use std::path::Path;
use png;
use super::{
    pipeline::AshPipeline,
    swapchain::AshSwapchain,
    context::AshContext,
};
use crate::adel_renderer_ash::definitions::{TriangleComponent, UniformBufferObject, Vertex, Vertex2d};
use super::structures;
use super::constants::MAX_FRAMES_IN_FLIGHT;
pub struct AshBuffers {
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    transient_command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    transient_queue: vk::Queue,

    // Temporary
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
}
impl AshBuffers {
    pub fn new(device: &ash::Device, context: &AshContext, swapchain: &AshSwapchain, pipeline: &AshPipeline) -> Self {
        let framebuffers = AshBuffers::create_framebuffers(
            &device,
            pipeline.render_pass().clone(),
            &swapchain.image_views,
            swapchain.extent()
        );
        let command_pool = AshBuffers::create_command_pool(&device, &context.queue_family, vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let transient_command_pool = AshBuffers::create_command_pool(&device, &context.queue_family, vk::CommandPoolCreateFlags::TRANSIENT);
        let (uniform_buffers, uniform_buffers_memory) = AshBuffers::create_uniform_buffers(&context, &device);
        let descriptor_pool = AshBuffers::create_descriptor_pool(&device);
        let descriptor_sets = AshBuffers::create_descriptor_sets(&device, descriptor_pool, pipeline.descriptor_set_layout(), &uniform_buffers);
        let command_buffers = AshBuffers::create_command_buffers(&device, command_pool);
        Self {
            framebuffers,
            command_pool,
            transient_command_pool,
            command_buffers,
            descriptor_pool,
            descriptor_sets,
            transient_queue: swapchain.graphics_queue.clone(),
            uniform_buffers,
            uniform_buffers_memory,
        }
    }
    fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        swapchain_extent: vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        let mut framebuffers = vec![];

        for &image_view in image_views.iter() {
            let attachments = [image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1)
                .build();

            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            framebuffers.push(framebuffer);
        }

        framebuffers
    }

    fn create_command_pool(
        device: &ash::Device,
        queue_families: &structures::QueueFamilyIndices,
        flags: vk::CommandPoolCreateFlags,
    ) -> vk::CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(flags)
            .queue_family_index(queue_families.graphics_family.unwrap())
            .build();

        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool!")
        }
    }
    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        }
    }
    fn create_descriptor_pool(
        device: &ash::Device
    ) -> vk::DescriptorPool {
        let pool_size = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
            .build()
        ];
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_size)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32)
            .build();
        unsafe {
            device.create_descriptor_pool(&descriptor_pool_create_info, None)
            .expect("Failed to create descriptor pool")
        }
    }
    pub fn create_descriptor_sets(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<vk::Buffer>
    ) -> Vec<vk::DescriptorSet> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = Vec::new();
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            layouts.push(descriptor_set_layout);
        }
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts)
            .build();

        let descriptor_sets = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to create DescriptorSets")
        };
        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = [vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .range(std::mem::size_of::<UniformBufferObject>() as u64)
                .offset(0)
                .build()];
            let descriptor_write_sets = [vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_array_element(0)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&descriptor_buffer_info)
                .build()];
            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }

        }
        descriptor_sets
    }
    pub fn recreate_framebuffers(&mut self, device: &ash::Device, render_pass: vk::RenderPass, image_views: &Vec<vk::ImageView>, extent: vk::Extent2D) {
        let framebuffers = AshBuffers::create_framebuffers(
            device,
            render_pass,
            image_views,
            extent,
        );
        self.framebuffers = framebuffers;
    }
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

    pub fn framebuffers(&self) -> &Vec<vk::Framebuffer> {
        &self.framebuffers
    }

    pub fn commandbuffers(&self) -> &Vec<vk::CommandBuffer> {
        &self.command_buffers
    }
    pub fn create_buffer(context: &AshContext, device: &ash::Device, device_size: vk::DeviceSize,
        usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags)
        -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(device_size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            //.queue_family_indices(0)
            .build();

        let buffer = unsafe {
            device
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to create Buffer")
        };
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let mem_properties =
            unsafe { context.instance.get_physical_device_memory_properties(context.physical_device) };
        let memory_type = find_memory_type(
            mem_requirements.memory_type_bits,
            properties,
            mem_properties,
        );

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type)
            .build();

        let buffer_memory = unsafe {
            device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate vertex buffer memory!")
        };

        unsafe {
            device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind Buffer");
        }
        (buffer, buffer_memory)
    }


    fn copy_buffer(&self, device: &ash::Device, src_buffer: &vk::Buffer, dst_buffer: &vk::Buffer, size: vk::DeviceSize) {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.transient_command_pool)
            .command_buffer_count(1)
            .build();

        let command_buffers = unsafe {
            device.allocate_command_buffers(&alloc_info).expect("Failed to allocate command buffer")
        };
        assert_eq!(command_buffers.len(), 1);
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        let copy_region = [vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(size)
            .build()];

        unsafe {
            device.begin_command_buffer(command_buffers[0], &begin_info).expect("Failed to begin command buffer");
            device.cmd_copy_buffer(command_buffers[0], *src_buffer, *dst_buffer, &copy_region);
            device.end_command_buffer(command_buffers[0]).expect("Failed to begin command buffer");
        };

        // Now that the command buffer has the copy command loaded, execute it
        let submit_info = [vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            // Fences may also be used here to submit multiple commands at once
            device.queue_submit(self.transient_queue, &submit_info, vk::Fence::null()).expect("Failed to submit copy buffer to queue");
            device.queue_wait_idle(self.transient_queue).expect("Failed for device to wait idle");
            device.free_command_buffers(self.transient_command_pool, &command_buffers);
        }
    }
    pub fn create_vertex_buffer(&self, context: &AshContext, device: &ash::Device, vertices :&Vec<Vertex>)
        -> (vk::Buffer, vk::DeviceMemory) {
        //let buffer_size = std::mem::size_of_val(&triangle.verticies) as vk::DeviceSize;
        let buffer_size = (vertices.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize;
        //log::info!("JAKOB buffer 1 {:?} buffer 2 {:?}", buffer_size, buffer_size2);
        let (staging_buffer, staging_buffer_memory) = AshBuffers::create_buffer(context, device, buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            device.unmap_memory(staging_buffer_memory);
        }
        let (vertex_buffer, vertex_buffer_memory) = AshBuffers::create_buffer(context, device, buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        self.copy_buffer(device, &staging_buffer, &vertex_buffer, buffer_size);

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (vertex_buffer, vertex_buffer_memory)
    }
    pub fn create_uniform_buffers(context: &AshContext, device: &ash::Device)
        -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>)
    {
        let buffer_size: vk::DeviceSize = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;
        let mut uniform_buffers: Vec<vk::Buffer> = Vec::new();
        let mut uniform_buffers_memory: Vec<vk::DeviceMemory> = Vec::new();
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (uniform_buffer, uniform_buffer_memory) = AshBuffers::create_buffer(
                context,
                device,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }

        (uniform_buffers, uniform_buffers_memory)

    }
    pub fn create_vertex_buffer2d(&self, context: &AshContext, device: &ash::Device, vertices :&Vec<Vertex2d>)
        -> (vk::Buffer, vk::DeviceMemory) {
        //let buffer_size = std::mem::size_of_val(&triangle.verticies) as vk::DeviceSize;
        let buffer_size = (vertices.len() * std::mem::size_of::<Vertex2d>()) as vk::DeviceSize;
        //log::info!("JAKOB buffer 1 {:?} buffer 2 {:?}", buffer_size, buffer_size2);
        let (staging_buffer, staging_buffer_memory) = AshBuffers::create_buffer(context, device, buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut Vertex2d;

            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            device.unmap_memory(staging_buffer_memory);
        }
        let (vertex_buffer, vertex_buffer_memory) = AshBuffers::create_buffer(context, device, buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        self.copy_buffer(device, &staging_buffer, &vertex_buffer, buffer_size);

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (vertex_buffer, vertex_buffer_memory)
    }



    pub fn create_index_buffer(&self, context: &AshContext, device: &ash::Device, indicies: &Vec<u16>) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_size = (indicies.len() * std::mem::size_of::<u16>()) as vk::DeviceSize;
        //log::info!("JAKOB buffer 1 {:?} buffer 2 {:?}", buffer_size, buffer_size2);
        let (staging_buffer, staging_buffer_memory) = AshBuffers::create_buffer(context, device, buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u16;

            data_ptr.copy_from_nonoverlapping(indicies.as_ptr(), indicies.len());

            device.unmap_memory(staging_buffer_memory);
        }
        let (index_buffer, index_buffer_memory) = AshBuffers::create_buffer(context, device, buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        self.copy_buffer(device, &staging_buffer, &index_buffer, buffer_size);

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (index_buffer, index_buffer_memory)
    }
    pub fn update_uniform_buffer(&self, device: &ash::Device, current_image: usize) {
        let ubos = [UniformBufferObject {
            model: nalgebra::Matrix4::<f32>::identity(),
            view: nalgebra::Matrix4::<f32>::identity(),
            proj: nalgebra::Matrix4::<f32>::identity(),
        }];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr =
                device
                    .map_memory(
                        self.uniform_buffers_memory[current_image],
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            device
                .unmap_memory(self.uniform_buffers_memory[current_image]);
        }
    }
    pub fn create_texture_image(context: &AshContext, device: &ash::Device, submit_queue: vk::Queue, image_path: &Path)
        -> (vk::Image, vk::DeviceMemory)
    {
        let image_file = std::fs::File::open(image_path).unwrap();
        let decoder = png::Decoder::new(image_file);
        let mut reader = decoder.read_info().unwrap();

        let mut pixels = vec![0; reader.info().raw_bytes()];
        reader.next_frame(&mut pixels).unwrap();
        let size = reader.info().raw_bytes() as u64;
        let (width, height) = reader.info().size();
        let (staging_buffer, staging_buffer_memory) = AshBuffers::create_buffer(
            context,
            device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u8;

            data_ptr.copy_from_nonoverlapping(pixels.as_ptr(), size as usize);

            device.unmap_memory(staging_buffer_memory);
        }
        AshBuffers::create_image(
            context,
            device,
            width,
            height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL
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
    ) -> (vk::Image, vk::DeviceMemory) {
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
            device.create_image(&info, None).expect("Failed to create image")
        };

        let tex_mem_requirements = unsafe {
            device.get_image_memory_requirements(texture_image)
        };
        let tex_mem_properties =
            unsafe { context.instance.get_physical_device_memory_properties(context.physical_device) };
        let tex_memory_type = find_memory_type(
            tex_mem_requirements.memory_type_bits,
            properties,
            tex_mem_properties,
        );
        let info = vk::MemoryAllocateInfo::builder()
            .allocation_size(tex_mem_requirements.size)
            .memory_type_index(tex_memory_type)
            .build();

        let texture_image_memory = unsafe {
            device.allocate_memory(&info, None).expect("Failed to allocate Texture Image memroy")
        };
        unsafe {
            device.bind_image_memory(texture_image, texture_image_memory, 0).expect("Failed to bind Texture to Memory");
        }

        (texture_image, texture_image_memory)
    }

    pub fn destroy_uniform_buffers(&mut self, device: &ash::Device) {
        unsafe{
            self.uniform_buffers.iter().for_each(|b| device.destroy_buffer(*b, None) );
            self.uniform_buffers_memory.iter().for_each(|m| device.free_memory(*m, None));
        }
    }
    pub fn destroy_descriptor_pool(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }

}

pub fn create_vertex_buffer_from_triangle(
    context: &AshContext,
    device: &ash::Device,
    triangle: &TriangleComponent
) -> (vk::Buffer, vk::DeviceMemory) {
    let vertex_buffer_create_info = vk::BufferCreateInfo::builder()
        .size(std::mem::size_of_val(&triangle.verticies) as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        //.queue_family_indices(0)
        .build();
    let vertex_buffer = unsafe {
        device
            .create_buffer(&vertex_buffer_create_info, None)
            .expect("Failed to create Vertex Buffer")
    };
    let mem_requirements = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };
    let mem_properties =
        unsafe { context.instance.get_physical_device_memory_properties(context.physical_device) };
    let required_memory_flags: vk::MemoryPropertyFlags =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let memory_type = find_memory_type(
        mem_requirements.memory_type_bits,
        required_memory_flags,
        mem_properties,
    );

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type)
        .build();

    let vertex_buffer_memory = unsafe {
        device
            .allocate_memory(&allocate_info, None)
            .expect("Failed to allocate vertex buffer memory!")
    };

    unsafe {
        device
            .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
            .expect("Failed to bind Buffer");

        let data_ptr = device
            .map_memory(
                vertex_buffer_memory,
                0,
                vertex_buffer_create_info.size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to Map Memory") as *mut Vertex2d;

        data_ptr.copy_from_nonoverlapping(triangle.verticies.as_ptr(), triangle.verticies.len());

        device.unmap_memory(vertex_buffer_memory);
    }

    (vertex_buffer, vertex_buffer_memory)


}

pub fn create_vertex_buffer_bak(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    vertices_data: &Vec<Vertex2d>
) -> (vk::Buffer, vk::DeviceMemory) {
    let vertex_buffer_create_info = vk::BufferCreateInfo::builder()
        .size(std::mem::size_of_val(vertices_data) as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        //.queue_family_indices(0)
        .build();
    let vertex_buffer = unsafe {
        device
            .create_buffer(&vertex_buffer_create_info, None)
            .expect("Failed to create Vertex Buffer")
    };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };
    let mem_properties =
        unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let required_memory_flags: vk::MemoryPropertyFlags =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let memory_type = find_memory_type(
        mem_requirements.memory_type_bits,
        required_memory_flags,
        mem_properties,
    );

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type)
        .build();

    let vertex_buffer_memory = unsafe {
        device
            .allocate_memory(&allocate_info, None)
            .expect("Failed to allocate vertex buffer memory!")
    };

    unsafe {
        device
            .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
            .expect("Failed to bind Buffer");

        let data_ptr = device
            .map_memory(
                vertex_buffer_memory,
                0,
                vertex_buffer_create_info.size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to Map Memory") as *mut Vertex2d;

        data_ptr.copy_from_nonoverlapping(vertices_data.as_ptr(), vertices_data.len());

        device.unmap_memory(vertex_buffer_memory);
    }

    (vertex_buffer, vertex_buffer_memory)
}

pub fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        //if (type_filter & (1 << i)) > 0 && (memory_type.property_flags & required_properties) == required_properties {
        //    return i as u32
        // }

        // same implementation
        if (type_filter & (1 << i)) > 0
            && memory_type.property_flags.contains(required_properties)
        {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type!")
}