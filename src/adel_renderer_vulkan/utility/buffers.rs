use ash::vk;
use super::structures;

pub fn create_framebuffers(
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

pub fn create_command_pool(
    device: &ash::Device,
    queue_families: &structures::QueueFamilyIndices,
) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_families.graphics_family.unwrap())
        .build();

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}

pub fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    graphics_pipeline: vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    surface_extent: vk::Extent2D,
    vertex_buffer: vk::Buffer,
    vertices_len: u32
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .command_buffer_count(framebuffers.len() as u32)
        .level(vk::CommandBufferLevel::PRIMARY)
        .build();

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };
    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE)
            .build();

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .render_area(vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0})
                .extent(surface_extent)
                .build()
            ).clear_values(&clear_values)
            .framebuffer(framebuffers[i])
            .build();
        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
            let vertex_buffers = [vertex_buffer];
            let offsets = [0_u64];

            device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);

            device.cmd_draw(command_buffer, vertices_len, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);

            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }
    command_buffers

}
pub fn record_command_buffers(
    device: &ash::Device,
    command_buffers: &Vec<vk::CommandBuffer>,
    render_pass: vk::RenderPass,
    surface_extent: vk::Extent2D,
    graphics_pipeline: vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>
) {

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE)
            .build();

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .render_area(vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0})
                .extent(surface_extent)
                .build()
            ).clear_values(&clear_values)
            .framebuffer(framebuffers[i])
            .build();

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
            log::info!("JAKOB this is where the error occurs");
            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);

            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

}
pub fn create_sync_objects(device: &ash::Device, max_frame_in_flight: usize) -> structures::SyncObjects {
    let mut sync_objects = structures::SyncObjects {
        image_available_semaphores: vec![],
        render_finished_semaphores: vec![],
        inflight_fences: vec![],
    };

    let semaphore_create_info = vk::SemaphoreCreateInfo::builder()
        .build();

    let fence_create_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED)
        .build();

    for _ in 0..max_frame_in_flight {
        unsafe {
            let image_available_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let render_finished_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let inflight_fence = device
                .create_fence(&fence_create_info, None)
                .expect("Failed to create Fence Object!");

            sync_objects
                .image_available_semaphores
                .push(image_available_semaphore);
            sync_objects
                .render_finished_semaphores
                .push(render_finished_semaphore);
            sync_objects.inflight_fences.push(inflight_fence);
        }
    }

    sync_objects
}
pub fn create_vertex_buffer(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    vertices_data: &Vec<structures::Vertex>
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
            .expect("Failed to Map Memory") as *mut structures::Vertex;

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