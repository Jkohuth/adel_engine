
use ash::vk;
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use log;
use std::ffi::{CString};
use std::os::raw::c_char;
use std::ptr;

use inline_spirv::include_spirv;
// TODO: Create a prelude and add these to it
use crate::adel_renderer_vulkan::utility::{
    constants::*,
    debug,
    platforms,
    structures,
    tools,
    swapchain,
    functions,
};
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};

pub struct VulkanApp {
    // vulkan stuff
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    _physical_device: vk::PhysicalDevice,
    device: ash::Device,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_info: structures::SwapChainInfo,
    swapchain_imageviews: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,
    graphics_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,

    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    window: winit::window::Window,
}

impl VulkanApp {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_title("Test Window")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(event_loop)
            .expect("Failed: Create window");

        // init vulkan stuff
        let entry = unsafe { ash::Entry::load().expect("Error: Failed to create Ash Entry") };
        let instance = functions::create_instance(&entry, ENABLE_VALIDATION_LAYERS, &VALIDATION_LAYERS.to_vec());
        let (debug_utils_loader, debug_messenger) = debug::setup_debug_utils(ENABLE_VALIDATION_LAYERS, &entry, &instance);
        let surface_info = functions::create_surface(&entry, &instance, &window);
        let physical_device = functions::pick_physical_device(&instance, &surface_info);
        let (device, family_indices) = create_logical_device(&instance, physical_device, &surface_info, &VALIDATION_LAYERS.to_vec());
        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        let swapchain_info = create_swapchain(
                                &instance,
                                &device,
                                physical_device,
                                &window,
                                &surface_info,
                                &family_indices);
        let swapchain_imageviews = create_image_views(&device, swapchain_info.swapchain_format, &swapchain_info.swapchain_images);

        let render_pass = create_render_pass(&device, swapchain_info.swapchain_format);

        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(&device, render_pass.clone(), swapchain_info.swapchain_extent);

        let framebuffers = create_framebuffers(&device, render_pass.clone(), &swapchain_imageviews, swapchain_info.swapchain_extent);

        let command_pool = create_command_pool(&device, &family_indices);
        let command_buffers = create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &framebuffers,
            render_pass,
            swapchain_info.swapchain_extent,
        );
        let sync_objects = create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        Self {
            _entry: entry,
            instance,
            surface_loader: surface_info.surface_loader,
            surface: surface_info.surface,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
            device,
            graphics_queue,
            present_queue,
            swapchain_info,
            swapchain_imageviews,
            render_pass,
            graphics_pipeline,
            pipeline_layout,
            framebuffers,
            command_pool,
            command_buffers,
            image_available_semaphores: sync_objects.image_available_semaphores,
            render_finished_semaphores: sync_objects.render_finished_semaphores,
            in_flight_fences: sync_objects.inflight_fences,
            current_frame: 0,
            window,
        }

    }

}
pub fn create_render_pass(device: &ash::Device, surface_format: vk::Format) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(surface_format)
        .flags(vk::AttachmentDescriptionFlags::empty())
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();


        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&[color_attachment_ref])
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

            let render_pass_attachments = [color_attachment];

        let subpass_dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dependency_flags(vk::DependencyFlags::empty())
            .build()];


        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies)
            .build();

        unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        }

}
pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_info: &structures::SurfaceInfo,
    required_validation_layers: &Vec<&str>
) ->  (ash::Device, structures::QueueFamilyIndices) {
    let indices = functions::find_queue_family(instance, physical_device, surface_info);

    use std::collections::HashSet;
    let mut unique_queue_familes = HashSet::new();
    unique_queue_familes.insert(indices.graphics_family.unwrap());
    unique_queue_familes.insert(indices.present_family.unwrap());

    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];
    for &queue_family in unique_queue_familes.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .flags(vk::DeviceQueueCreateFlags::empty())
        .queue_family_index(queue_family)
        .queue_priorities(&queue_priorities)
        .build();
        queue_create_infos.push(queue_create_info);
    }
    let physical_device_features = vk::PhysicalDeviceFeatures::builder().build();

    let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let layer_names: Vec<*const i8> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

        let enabled_extension_names = DEVICE_EXTENSIONS.get_extensions_raw_names();


    let device_create_info = if ENABLE_VALIDATION_LAYERS {
     vk::DeviceCreateInfo::builder()
        .flags(vk::DeviceCreateFlags::empty())
        .queue_create_infos(&queue_create_infos)
        .enabled_layer_names(&layer_names)
        .enabled_extension_names(&enabled_extension_names)
        .enabled_features(&physical_device_features)
        .build()
    } else {
        vk::DeviceCreateInfo::builder()
        .flags(vk::DeviceCreateFlags::empty())
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&enabled_extension_names)
        .enabled_features(&physical_device_features)
        .build()
    };
    let device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("ERROR: Failed to create logical device")
    };

    (device, indices)
}


pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    window: &winit::window::Window,
    surface_info: &structures::SurfaceInfo,
    queue_family: &structures::QueueFamilyIndices,
) -> structures::SwapChainInfo {
    let swapchain_support = swapchain::query_swapchain_support(physical_device, surface_info);

    let surface_format = swapchain::choose_swapchain_format(&swapchain_support.formats);
    let present_mode = swapchain::choose_swapchain_present_mode(&swapchain_support.present_modes);
    let extent = swapchain::choose_swapchain_extent(&swapchain_support.capabilities, window);

    let image_count = swapchain_support.capabilities.min_image_count + 1;
    let image_count = if swapchain_support.capabilities.max_image_count > 0 {
        image_count.min(swapchain_support.capabilities.max_image_count)
    } else {
        image_count
    };

    let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
        if queue_family.graphics_family != queue_family.present_family {
            (
                vk::SharingMode::CONCURRENT,
                2,
                vec![
                    queue_family.graphics_family.unwrap(),
                    queue_family.present_family.unwrap(),
                ],
            )
        } else {
            (vk::SharingMode::EXCLUSIVE, 0, vec![])
        };
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface_info.surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_array_layers(1)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(swapchain_support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .build();

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("ERROR: Failed to create swapchain")
    };
    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("ERROR: Failed to get swapchain images")
    };

    structures::SwapChainInfo {
        swapchain_loader,
        swapchain,
        swapchain_format: surface_format.format,
        swapchain_extent: extent,
        swapchain_images,
    }
}

pub fn create_image_views(
    device: &ash::Device,
    surface_format: vk::Format,
    images: &Vec<vk::Image>,
) -> Vec<vk::ImageView> {
    let swapchain_imagesviews: Vec<vk::ImageView> = images.iter()
        .map(|&image| {
            create_image_view(
                device,
                image,
                surface_format,
                vk::ImageAspectFlags::COLOR,
                1
            )
        }).collect();

    swapchain_imagesviews
}
pub fn create_image_view(
    device: &ash::Device,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    mip_levels: u32,
) -> vk::ImageView {
    let imageview_create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .components(vk::ComponentMapping::builder()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY)
            .build())
        .subresource_range(vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_flags)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1)
            .build())
        .build();


    unsafe {
        device
            .create_image_view(&imageview_create_info, None)
            .expect("Failed to create Image View!")
    }
}
pub fn create_shader_module(device: &ash::Device, code: &[u32]) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(&code).build();

    // Call to graphics card to build shader
    unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create Shader Module!")
    }
}
pub fn create_graphics_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
) -> (vk::Pipeline, vk::PipelineLayout) {

    // Create Shader Modules
    let vert_spv: &'static [u32] = include_spirv!("src/adel_renderer_vulkan/shaders/triangle.vert", vert, glsl, entry="main");
    let frag_spv: &'static [u32] = include_spirv!("src/adel_renderer_vulkan/shaders/triangle.frag", frag, glsl, entry="main");
    let vert_shader = create_shader_module(&device, vert_spv);
    let frag_shader = create_shader_module(&device, frag_spv);


    let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::builder()
            .module(vert_shader)
            .name(&main_function_name)
            .stage(vk::ShaderStageFlags::VERTEX)
            .build(),
        vk::PipelineShaderStageCreateInfo::builder()
            .module(frag_shader)
            .name(&main_function_name)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .build()
    ];
    let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription::builder()
        .binding(0)
        .stride(std::mem::size_of::<Vertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX)
        .build()
    ];
    let vertex_input_attribute_descriptions = [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex, position) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, color) as u32)
                .build()
            ];
    let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_input_binding_descriptions)
        .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
        .build();

    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build();

    let viewports = [vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_extent.width as f32)
        .height(swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build()
    ];

    let scissors = [vk::Rect2D::builder()
        .offset(vk::Offset2D::builder()
                    .x(0).y(0).build())
        .extent(swapchain_extent)
        .build()];

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissor_count(scissors.len() as u32)
        .scissors(&scissors)
        .viewport_count(viewports.len() as u32)
        .viewports(&viewports)
        .build();

    let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0)
        .polygon_mode(vk::PolygonMode::FILL)
        .rasterizer_discard_enable(false)
        .depth_bias_clamp(0.0)
        .depth_bias_constant_factor(0.0)
        .depth_bias_enable(false)
        .depth_bias_slope_factor(0.0)
        .build();

    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

    let stencil_state = vk::StencilOpState::builder()
        .fail_op(vk::StencilOp::KEEP)
        .pass_op(vk::StencilOp::KEEP)
        .depth_fail_op(vk::StencilOp::KEEP)
        .compare_op(vk::CompareOp::ALWAYS)
        .compare_mask(0)
        .write_mask(0)
        .reference(0)
        .build();

    let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false)
        .front(stencil_state)
        .back(stencil_state)
        .max_depth_bounds(1.0)
        .min_depth_bounds(0.0)
        .build();

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ZERO)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build()
    ];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachment_states)
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build();

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder().build();

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Failed to create pipeline layout!")
    };

    let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state_create_info)
        .input_assembly_state(&vertex_input_assembly_state_info)
        .viewport_state(&viewport_state_create_info)
        .rasterization_state(&rasterization_statue_create_info)
        .multisample_state(&multisample_state_create_info)
        .depth_stencil_state(&depth_state_create_info)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::null())
        .base_pipeline_index(-1)
        .build()
    ];

    log::info!("Creating the graphics pipeline");
    let graphics_pipelines = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &graphic_pipeline_create_infos,
                None,
            )
            .expect("Failed to create Graphics Pipeline!.")
    };

    unsafe {
        device.destroy_shader_module(vert_shader, None);
        device.destroy_shader_module(frag_shader, None);
    }

    (graphics_pipelines[0], pipeline_layout)
}

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
/*
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
            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);

            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }*/

    command_buffers
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
impl VulkanApp {
    fn draw_frame(&mut self) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            self.swapchain_info.swapchain_loader
                .acquire_next_image(
                    self.swapchain_info.swapchain,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
        };

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[self.command_buffers[image_index as usize]])
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain_info.swapchain];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&[image_index])
            .build();

        unsafe {
            self.swapchain_info.swapchain_loader
                .queue_present(self.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
    pub fn main_loop(mut self, event_loop: EventLoop<()>) {

        event_loop.run(move |event, _, control_flow| {

            match event {
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                | KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        | _ => {},
                                    }
                                },
                            }
                        },
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => {
                    self.window.request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    self.draw_frame();
                },
                | Event::LoopDestroyed => {
                    unsafe {
                        self.device.device_wait_idle()
                            .expect("Failed to wait device idle!")
                    };
                },
                _ => (),
            }

        })
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }

            self.swapchain_info.swapchain_loader
                .destroy_swapchain(self.swapchain_info.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);

            if ENABLE_VALIDATION_LAYERS {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
struct Vertex {
    position: nalgebra::Vector2<f32>,
    color: nalgebra::Vector3<f32>
}