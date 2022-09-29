use ash::vk;
use std::ffi::CString;
use inline_spirv::include_spirv;

use super::structures::{PushConstantData, Vertex};

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
    //let vert_spv: &'static [u32] = include_spirv!("src/adel_renderer_vulkan/shaders/triangle.vert", vert, glsl, entry="main");
    //let frag_spv: &'static [u32] = include_spirv!("src/adel_renderer_vulkan/shaders/triangle.frag", frag, glsl, entry="main");
    let vert_spv: &'static [u32] = include_spirv!("src/adel_renderer_vulkan/shaders/push.vert", vert, glsl, entry="main");
    let frag_spv: &'static [u32] = include_spirv!("src/adel_renderer_vulkan/shaders/push.frag", frag, glsl, entry="main");
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

    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let pipeline_dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_state)
        .build();
    let push_constant_range = [vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(std::mem::size_of::<PushConstantData>() as u32)
        .build()];
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
        .push_constant_ranges(&push_constant_range)
        .build();

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
        .dynamic_state(&pipeline_dynamic_state)
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
