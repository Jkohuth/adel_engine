
use std::sync::Arc;
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

use crate::adel_renderer::FinalImageView;

use crate::adel_renderer::{
    VulkanoContext,
    VulkanoWindow,
    VulkanoPipeline,
    Vertex,
    Vertex2d,
    ModelComponent,
    TransformComponent,
    Transform2dComponent,
    vs_push::ty::PushConstantData,
};
use crate::adel_ecs::{System, World};

use glam::{Mat4, Vec3};

pub struct VulkanoRenderer {
    context: VulkanoContext,
    window: VulkanoWindow,
    pipeline: VulkanoPipeline,
    name: &'static str,
}

impl VulkanoRenderer {
    pub fn new(window: Window) -> Self {
        let context = VulkanoContext::new();
        let window = VulkanoWindow::new(&context, window);
        
        /*if context.graphics_queue().family().supports_surface(&window.surface()).unwrap() {
            log::info!("Present Queue supported by graphics queue");
        }*/

        let pipeline = VulkanoPipeline::new(context.device(), window.render_pass());
        
        //log::info!("View Matrix {:?}", &camera.get_view());
        // Move this up further in the build
        window.window().set_visible(true);

        let name = "Renderer";
        Self {
            context,
            window,
            pipeline,
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
    pub fn vulkano_pipeline(&self) -> &VulkanoPipeline {
        &self.pipeline
    }


    pub fn render(&mut self, data: Vec::<(Arc<CpuAccessibleBuffer<[Vertex]>>, PushConstantData)>) {
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
            .bind_pipeline_graphics(self.pipeline.pipeline().clone());

        for i in data {
            builder
                .bind_vertex_buffers(0, i.0.clone())
                .push_constants(self.pipeline.pipeline().layout().clone(), 0, i.1.clone())
                .draw(i.0.len() as u32, 1, 0, 0).unwrap();
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

    /*fn create_vertex_buffers_2d(&self, verticies: Vec<Vertex2d>) -> Result<Arc<CpuAccessibleBuffer<[Vertex2d]>>, DeviceMemoryAllocationError> {
        CpuAccessibleBuffer::from_iter(
            self.context.device().clone(), 
            BufferUsage::all(), 
            false,
            verticies)
    }*/

    fn create_vertex_buffers(&self, verticies: Vec<Vertex>) -> Result<Arc<CpuAccessibleBuffer<[Vertex]>>, DeviceMemoryAllocationError> {
        CpuAccessibleBuffer::from_iter(
            self.context.device().clone(), 
            BufferUsage::all(), 
            false,
            verticies)
    }
}
/*
// Temporary solution - Do nothing to it such that it can compile
fn create_push_constant_data_2d(transform: &Transform2dComponent) -> PushConstantData2d {
    PushConstantData2d {
        transform: transform.mat2().into(),
        offset: transform.translation.into(),
        color: [0.0, 0.0, 0.0],
        _dummy0: [0,0,0,0,0,0,0,0],
    }
}
*/

fn create_push_constant_data(camera_projection: Mat4, transform: &TransformComponent) -> PushConstantData {
    PushConstantData { 
        transform: (camera_projection * transform.mat4_less_computation()).to_cols_array_2d(),
        color: [0.0, 0.0, 0.0],
    }
}
impl System for VulkanoRenderer {
    fn run(&mut self, world: &mut World) {
        let camera = world.get_resource::<Camera>().unwrap();
        let projection_matrix = camera.get_projection() * camera.get_view();
        let mut model_ref = world.borrow_component_mut::<ModelComponent>().unwrap();
        let mut transform_ref = world.borrow_component_mut::<TransformComponent>().unwrap(); 
        let mut data = Vec::<(Arc<CpuAccessibleBuffer<[Vertex]>>, PushConstantData)>::new();
        // Retrive a tuple, (usize, Value)
        for i in model_ref.iter_mut().enumerate() {
            match i.1 {
                Some(model) => {
                    if let Some(transform) = &mut transform_ref[i.0] {
                        transform.rotation.y = &transform.rotation.y + 0.01 % std::f32::consts::PI;
                        transform.rotation.x = &transform.rotation.x + 0.005 % std::f32::consts::PI;
                        data.push( (self.create_vertex_buffers(model.verticies.clone()).unwrap(), 
                            create_push_constant_data(projection_matrix.clone(), &transform)));
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