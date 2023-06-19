use crate::adel_tools::as_bytes;
use crate::renderer::UniformBufferObject;
use crate::{
    adel_renderer::{
        definitions::{PointLightPushConstants, TransformComponent},
        utility::{descriptors::AshDescriptors, pipeline::AshPipeline},
    },
    renderer::PointLightComponent,
};
use std::cell::{Ref, RefMut};

use crate::adel_renderer::{vec3_to_vec4, vec4_to_vec3};
use anyhow::Result;
use ash::vk;
use inline_spirv::include_spirv;
use nalgebra::Vector4;
use std::ffi::CString;

pub struct PointLightRenderer {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

impl PointLightRenderer {
    pub fn new(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<Self> {
        let pipeline_layout =
            PointLightRenderer::create_pipeline_layout(device, descriptor_set_layout)?;
        let pipeline =
            PointLightRenderer::create_pipeline(device, pipeline_layout, render_pass, extent)?;
        Ok(Self {
            pipeline_layout,
            pipeline,
        })
    }
    // TODO: Remove Mut from transform Component, its there for the update function
    pub fn render(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        descriptors: &AshDescriptors,
        point_light_entities: &Vec<usize>,
        point_lights: &Ref<Vec<Option<PointLightComponent>>>,
        transforms: &RefMut<Vec<Option<TransformComponent>>>,
    ) -> Result<()> {
        //let device_size_offsets: [vk::DeviceSize; 1] = [0];
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
            for entity in point_light_entities.iter() {
                let point_light = point_lights[*entity].unwrap();
                let transform = transforms[*entity].unwrap();
                let push: PointLightPushConstants = PointLightPushConstants {
                    position: Vector4::new(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                        1.0,
                    ),
                    color: point_light.color,
                    radius: transform.scale.x,
                };
                device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    as_bytes(&push),
                );
                device.cmd_draw(command_buffer, 6, 1, 0, 0);
            }
            for i in point_lights.iter() {}
        }
        Ok(())
    }
    pub fn update(
        dt: f32,
        point_light_entities: &Vec<usize>,
        point_lights: &Ref<Vec<Option<PointLightComponent>>>,
        transform_component: &mut RefMut<Vec<Option<TransformComponent>>>,
        ubo: &mut UniformBufferObject,
    ) -> Result<()> {
        let axis = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, -1.0, 0.0));
        let rotation = nalgebra::Matrix4::<f32>::identity()
            * nalgebra::Rotation3::from_axis_angle(&axis, 0.5 * dt).to_homogeneous();
        let mut light_index = 0;
        for entity in point_light_entities {
            if let Some(point_light) = point_lights[*entity] {
                if let Some(transform) = &mut transform_component[*entity] {
                    let translation = vec3_to_vec4(transform.translation);
                    transform.translation = vec4_to_vec3(rotation * translation);

                    // copy light to ubo
                    ubo.point_lights[light_index].position = Vector4::new(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                        1.0,
                    );
                    ubo.point_lights[light_index].color = Vector4::new(
                        point_light.color.x,
                        point_light.color.y,
                        point_light.color.z,
                        1.0,
                    );

                    light_index += 1;
                }
            }
            // update light position
        }
        ubo.num_lights = light_index as u32;
        Ok(())
    }
    fn create_pipeline_layout(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        let set_layouts = [descriptor_set_layout];
        let push_constant_range = [vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<PointLightPushConstants>() as u32)
            .build()];
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
            "src/adel_renderer/shaders/point_light.vert",
            vert,
            glsl,
            entry = "main"
        );
        let frag_spv: &'static [u32] = include_spirv!(
            "src/adel_renderer/shaders/point_light.frag",
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
        let vertex_input_state_create_info =
            vk::PipelineVertexInputStateCreateInfo::builder().build();
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
    pub unsafe fn destroy_point_light_renderer(&mut self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
}
