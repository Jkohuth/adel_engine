use std::sync::Arc;

#[allow(unused_imports)]
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents },
    device::{
        Device,
    },
    image::{
        swapchain::SwapchainImage, view::ImageView,
    },
    pipeline::{
        GraphicsPipeline,
        graphics::{
            depth_stencil::DepthStencilState,
            color_blend::ColorBlendState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::Viewport,
            viewport::ViewportState,
        },
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass },
    shader::ShaderModule,
    swapchain::{ Swapchain, },
};

use crate::adel_renderer::{Vertex, Vertex2d};
use crate::adel_renderer::PipelineType;

pub struct VulkanoPipeline {
    pipeline: Arc<GraphicsPipeline>,
    _vs: Arc<ShaderModule>,
    _fs: Arc<ShaderModule>,
    pipeline_type: PipelineType,

}

impl VulkanoPipeline {
pub fn new(device: &Arc<Device>, render_pass: &Arc<RenderPass>, pipeline_type: PipelineType) -> Self {
        //let _vs = crate::adel_vulkano::vs::load(device.clone()).unwrap();
        //let _fs = crate::adel_vulkano::fs::load(device.clone()).unwrap();
        match pipeline_type {
            PipelineType::Model => {

                let _vs = crate::adel_renderer::vs_push::load(device.clone()).unwrap();
                let _fs = crate::adel_renderer::fs_push::load(device.clone()).unwrap();

                let pipeline = Self::create_model_pipeline(
                    &device,
                    Subpass::from(render_pass.clone(), 0).unwrap(),
                    &_vs,
                    &_fs);
                return Self {
                    pipeline,
                    _vs,
                    _fs,
                    pipeline_type,
                };
            },
            PipelineType::Image => {
                let _vs = crate::adel_renderer::vs_texture::load(device.clone()).unwrap();
                let _fs = crate::adel_renderer::fs_texture::load(device.clone()).unwrap();

                let pipeline = Self::create_image_pipeline(
                    &device,
                    Subpass::from(render_pass.clone(), 0).unwrap(),
                    &_vs,
                    &_fs);
                return Self {
                    pipeline,
                    _vs,
                    _fs,
                    pipeline_type,
                }

            }
        };
    }

    fn create_model_pipeline(
        device: &Arc<Device>,
        subpass: Subpass,
        vs: &Arc<ShaderModule>,
        fs: &Arc<ShaderModule>,

    ) -> Arc<GraphicsPipeline> {
        GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        //.input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::TriangleList))
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(subpass)
        .build(device.clone())
        .unwrap()
    }

    fn create_image_pipeline(
        device: &Arc<Device>,
        subpass: Subpass,
        vs: &Arc<ShaderModule>,
        fs: &Arc<ShaderModule>,

    ) -> Arc<GraphicsPipeline> {
        GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex2d>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::TriangleList))
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(subpass)
        .build(device.clone())
        .unwrap()
    }

    pub fn pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }
}