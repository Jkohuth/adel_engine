
use std::sync::Arc;
use std::collections::HashMap;
use std::cell::{Ref, RefCell, RefMut};
use log;
#[allow(unused_imports)]
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess, },
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents },
    image::{ ImageAccess, swapchain::SwapchainImage, view::ImageView},
    memory::DeviceMemoryAllocationError,
    pipeline::graphics::viewport::Viewport,
    pipeline::Pipeline,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, },
    sync::{ GpuFuture },
};
use winit::window::Window;

use crate::adel_camera::Camera;

use crate::adel_renderer::{renderer_utils, FinalImageView};

use crate::adel_renderer::{
    VulkanoContext,
    VulkanoWindow,
    VulkanoPipeline,
    Vertex,
    Vertex2d,
    ModelBuilder,
    ModelComponent,
    TransformComponent,
    Transform2dComponent,
    vs_push::ty::PushConstantData,
};
use crate::adel_ecs::{System, World};

use glam::{Mat4, Vec3};

#[derive(PartialEq, Eq, Hash)]
pub enum PipelineType {
    Model,
    Image
}

pub struct VulkanoRenderer {
    context: VulkanoContext,
    window: VulkanoWindow,
    pipeline_map: HashMap<PipelineType, VulkanoPipeline>,
    name: &'static str,
}

impl VulkanoRenderer {
    pub fn new(window: Window) -> Self {
        let context = VulkanoContext::new();
        let window = VulkanoWindow::new(&context, window);

        /*if context.graphics_queue().family().supports_surface(&window.surface()).unwrap() {
            log::info!("Present Queue supported by graphics queue");
        }*/

        let model_pipeline = VulkanoPipeline::new(context.device(), window.render_pass(), PipelineType::Model);
        let image_pipeline = VulkanoPipeline::new(context.device(), window.render_pass(), PipelineType::Image);
        let mut pipeline_map: HashMap<PipelineType, VulkanoPipeline> = HashMap::new();
        pipeline_map.insert(PipelineType::Model, model_pipeline);
        pipeline_map.insert(PipelineType::Image, image_pipeline);

        //log::info!("View Matrix {:?}", &camera.get_view());
        // Move this up further in the build
        window.window().set_visible(true);

        let name = "Renderer";
        Self {
            context,
            window,
            pipeline_map,
            name,
        }
    }

    pub fn vulkano_window(&self) -> &VulkanoWindow {
        &self.window
    }
    pub fn vulkano_window_mut(&mut self) -> &mut VulkanoWindow {
        &mut self.window
    }
    pub fn vulkano_context(&self) -> &VulkanoContext {
        &self.context
    }
    pub fn vulkano_pipeline_map(&self) -> &HashMap<PipelineType, VulkanoPipeline> {
        &self.pipeline_map
    }
    // Requires the model builder components from World and returns a RefCell enclosed Vector where entity value
    // equates to it's position in the vector and if an entity has some value it's
    pub fn create_models(&self, model_builder_vec: &mut RefMut<Vec<Option<ModelComponent>>>) {
        for i in model_builder_vec.iter_mut().enumerate() {
            match i.1 {
                Some(builder) => {
                    builder.build(&self.context.device());
                } None => {
                    log::debug!("Entity {} does not have a model component", i.0);
                }
            }
        }
    }


    pub fn render(&mut self, data: Vec::<(Arc<CpuAccessibleBuffer<[Vertex]>>, Arc<CpuAccessibleBuffer<[u16]>>, PushConstantData)>) {
        // TODO: Very much not a fan of passing an Arc<Device> into every call of start frame
        let before_pipeline_future = match self.window.start_frame(self.context.device()) {
            Err(e) => {
                log::warn!("{}", e.to_string());
                return;
            }
            Ok(future) => future,
        };

        let clear_values = vec![Some([0.1, 0.1, 0.1, 1.0].into()), Some(1f32.into())];

        // TODO: Check into Vulkano find out if Command buffers can be made and store in advance
        let mut builder = AutoCommandBufferBuilder::primary(
            self.context.device().clone(),
            self.window.graphics_queue().family(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        // TODO: Make Secondary CommandBuffers - Load up all of the images onto the primary to do a single render
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values,
                    ..RenderPassBeginInfo::framebuffer(self.window.get_framebuffer().clone())
                },
                SubpassContents::Inline,
            ).unwrap()
            .set_viewport(0, [self.window.viewport().clone()])
            .bind_pipeline_graphics(self.pipeline_map.get(&PipelineType::Model).unwrap().pipeline().clone());

        for i in data {
            builder
                .bind_vertex_buffers(0, i.0.clone())
                .bind_index_buffer(i.1.clone())
                .push_constants(self.pipeline_map.get(&PipelineType::Model).unwrap().pipeline().layout().clone(), 0, i.2.clone())
                //.draw(i.0.len() as u32, 1, 0, 0).unwrap();
                .draw_indexed(i.1.len() as u32, 1, 0, 0, 0).unwrap();
        }
        builder
            .end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();

        let after_render = match before_pipeline_future.then_execute(self.window.graphics_queue(), command_buffer) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Error {:?}", e);
                panic!("GPU Future failed");
            }
        };

        let after_render = after_render.boxed();

        self.window.finish_frame(after_render);
    }

}

impl System for VulkanoRenderer {
    fn run(&mut self, world: &mut World) {

        // Need to build the models into Model Components

        let camera = world.get_resource::<Camera>().unwrap();
        let projection_matrix = camera.get_projection() * camera.get_view();
        let mut model_ref = world.borrow_component_mut::<ModelComponent>().unwrap();
        let mut transform_ref = world.borrow_component_mut::<TransformComponent>().unwrap();
        let mut data = Vec::<(Arc<CpuAccessibleBuffer<[Vertex]>>, Arc<CpuAccessibleBuffer<[u16]>>, PushConstantData)>::new();
        // Retrive a tuple, (usize, Value)
        for i in model_ref.iter_mut().enumerate() {
            match i.1 {
                Some(model) => {
                    if let Some(transform) = &mut transform_ref[i.0] {
                        //transform.rotation.y = &transform.rotation.y + 0.01 % std::f32::consts::PI;
                        //transform.rotation.x = &transform.rotation.x + 0.005 % std::f32::consts::PI;
                        data.push( (model.vertex_buffer.as_ref().unwrap().clone(), model.index_buffer.as_ref().unwrap().clone(),
                            renderer_utils::create_push_constant_data(projection_matrix.clone(), &transform))
                        );
                    }
                }
                None => (),
            }
        }
        self.render(data);
    }

    fn name(&self) -> &str {
        self.name
    }
}