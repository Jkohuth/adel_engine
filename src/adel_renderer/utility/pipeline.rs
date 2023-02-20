use crate::adel_renderer::definitions::{PushConstantData, Vertex};
use anyhow::Result;
use ash::vk::{self, GraphicsPipelineCreateInfo};
use inline_spirv::include_spirv;
use std::ffi::CString;

pub struct AshPipeline {
    descriptor_set_layout: vk::DescriptorSetLayout,
    graphics_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

impl AshPipeline {
    pub fn new(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<Self> {
        //let descriptor_set_layout = AshPipeline::create_descriptor_set_layout_ubo_texture(&device)?;
        let descriptor_set_layout = AshPipeline::create_descriptor_set_layout_ubo(&device)?;
        let pipeline_layout = AshPipeline::create_pipeline_layout(&device, descriptor_set_layout)?;
        /*let graphics_pipeline = AshPipeline::create_graphics_pipeline(
            &device,
            render_pass.clone(),
            pipeline_layout,
            extent,
        )?;*/
        //let (graphics_pipeline, pipeline_layout) = AshPipeline::create_graphics_pipeline(&device, render_pass.clone(), descriptor_set_layout, extent);
        todo!()
        /*Ok(Self {
            descriptor_set_layout,
            graphics_pipeline,
            pipeline_layout,
        })*/
    }

    #[allow(dead_code)]
    fn create_descriptor_set_layout_ubo(device: &ash::Device) -> Result<vk::DescriptorSetLayout> {
        let ubo_layout_bindings = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS)
            .build();
        let bindings = &[ubo_layout_bindings];
        let descriptor_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();
        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&descriptor_layout_info, None)? };
        Ok(descriptor_set_layout)
    }
    #[allow(dead_code)]
    fn create_descriptor_set_layout_ubo_texture(
        device: &ash::Device,
    ) -> Result<vk::DescriptorSetLayout> {
        let ubo_layout_bindings = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();
        let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();
        let bindings = &[ubo_layout_bindings, sampler_binding];
        let descriptor_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();
        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&descriptor_layout_info, None)? };

        Ok(descriptor_set_layout)
    }
    fn create_pipeline_layout(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        let push_constant_range = [vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<PushConstantData>() as u32)
            .build()];
        let set_layouts = [descriptor_set_layout];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&push_constant_range)
            .set_layouts(&set_layouts)
            .build();

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None)? };
        Ok(pipeline_layout)
    }
    pub fn create_shader_module(device: &ash::Device, code: &[u32]) -> Result<vk::ShaderModule> {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(&code).build();

        // Call to graphics card to build shader
        let shader_module =
            unsafe { device.create_shader_module(&shader_module_create_info, None)? };
        Ok(shader_module)
    }

    pub fn create_graphics_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        pipeline_layout: vk::PipelineLayout,
        swapchain_extent: vk::Extent2D,
        graphics_pipeline_create_info_builder: vk::GraphicsPipelineCreateInfoBuilder,
    ) -> Result<vk::Pipeline> {
        // Create Shader Modules
        /* let vert_spv: &'static [u32] = include_spirv!(
                            "src/adel_renderer/shaders/uniform_buffer.vert",
                            vert,
                            glsl,
                            entry = "main"
                        );
                        let frag_spv: &'static [u32] = include_spirv!(
                            "src/adel_renderer/shaders/uniform_buffer.frag",
                            frag,
                            glsl,
                            entry = "main"
                        );
                        let vert_spv: &'static [u32] = include_spirv!(
                            "src/adel_renderer/shaders/texture.vert",
                            vert,
                            glsl,
                            entry = "main"
                        );
                        let frag_spv: &'static [u32] = include_spirv!(
                            "src/adel_renderer/shaders/texture.frag",
                            frag,
                            glsl,
                            entry = "main"
                        );
                        let vert_shader = AshPipeline::create_shader_module(&device, vert_spv)?;
                        let frag_shader = AshPipeline::create_shader_module(&device, frag_spv)?;

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
                        .build(),
                ];
                let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
                    .vertex_binding_descriptions(&Vertex::binding_descriptions())
                    .vertex_attribute_descriptions(&Vertex::attribute_descriptions())
                    .build();

                let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
                    .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                    .primitive_restart_enable(false)
                    .build();
        */
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
            .build()];

        let scissors = [vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
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
            //.cull_mode(vk::CullModeFlags::NONE)
            //.front_face(vk::FrontFace::CLOCKWISE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
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
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state)
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
            .build()];
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

        /*let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state_create_info)
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
        .build()];*/
        let graphics_pipeline_create_infos = [graphics_pipeline_create_info_builder
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
            .build()];

        log::info!("Creating the graphics pipeline");
        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphics_pipeline_create_infos,
                    None,
                )
                .expect("Failed to create Graphics Pipeline")
        };
        log::info!("Created Graphics pipeline");
        Ok(graphics_pipelines[0])
    }

    pub fn graphics_pipeline(&self) -> vk::Pipeline {
        self.graphics_pipeline
    }
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
    pub unsafe fn destroy_descriptor_set_layout(&mut self, device: &ash::Device) {
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
    pub unsafe fn destroy_pipeline(&mut self, device: &ash::Device) {
        device.destroy_pipeline(self.graphics_pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
}
