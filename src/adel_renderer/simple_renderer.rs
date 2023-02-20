use crate::adel_renderer::definitions::PushConstantData;
use crate::adel_renderer::definitions::Vertex;
use crate::adel_renderer::utility::model::ModelComponent;
use crate::adel_renderer::utility::pipeline::AshPipeline;
use crate::adel_tools::as_bytes;
use anyhow::Result;
use ash::vk;
use inline_spirv::include_spirv;
use std::ffi::CString;

use super::utility::descriptors::AshDescriptors;

pub struct SimpleRenderer {
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl SimpleRenderer {
    pub fn new(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<Self> {
        let pipeline_layout =
            SimpleRenderer::create_pipeline_layout(device, descriptor_set_layout)?;
        let pipeline =
            SimpleRenderer::create_pipeline(device, pipeline_layout, render_pass, extent)?;
        Ok(Self {
            pipeline_layout,
            pipeline,
        })
    }
    pub fn render(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        models: Vec<(&ModelComponent, PushConstantData)>,
        frame_index: usize,
        descriptors: &AshDescriptors,
    ) -> Result<()> {
        let device_size_offsets: [vk::DeviceSize; 1] = [0];
        let descriptor_sets_to_bind = [descriptors.global_descriptor_sets[frame_index]];
        //let descriptor_sets = self.buffers.descriptor_sets.as_ref().unwrap();
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &descriptor_sets_to_bind,
                &[],
            );
            for model in models.iter() {
                device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &[model.0.vertex_buffer.buffer().clone()],
                    &device_size_offsets,
                );
                device.cmd_bind_index_buffer(
                    command_buffer,
                    model.0.index_buffer.buffer().clone(),
                    0,
                    vk::IndexType::UINT32,
                );
                device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    as_bytes(&model.1),
                );

                device.cmd_draw_indexed(command_buffer, model.0.indices_count, 1, 0, 0, 0);
            }
        }
        Ok(())
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
    fn create_pipeline(
        device: &ash::Device,
        pipeline_layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<vk::Pipeline> {
        // Shader Modules are unique to each Render System. They need to be generated, loaded up into pipeline builder and passed in
        let vert_spv: &'static [u32] = include_spirv!(
            "src/adel_renderer/shaders/model_renderer.vert",
            vert,
            glsl,
            entry = "main"
        );
        let frag_spv: &'static [u32] = include_spirv!(
            "src/adel_renderer/shaders/model_renderer.frag",
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
        let graphics_pipeline_builder = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_create_info);

        let graphics_pipeline = AshPipeline::create_graphics_pipeline(
            device,
            render_pass,
            pipeline_layout,
            extent,
            graphics_pipeline_builder,
        )?;
        log::info!("Created Model Graphics pipeline");
        unsafe {
            device.destroy_shader_module(vert_shader, None);
            device.destroy_shader_module(frag_shader, None);
        }
        Ok(graphics_pipeline)
    }

    pub fn graphics_pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
    pub unsafe fn destroy_simple_renderer(&mut self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
}
