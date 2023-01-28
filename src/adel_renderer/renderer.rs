
use ash::vk;
use anyhow::{anyhow, Result};
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use nalgebra::{Matrix4, Vector2, Vector3};
use log;

use crate::adel_ecs::{System, World};
// TODO: Create a prelude and add these to it
use crate::adel_renderer::utility::{
    constants::*,
    swapchain::AshSwapchain,
    context::{AshContext, create_logical_device},
    pipeline::AshPipeline,
    buffers::AshBuffers,
    sync::SyncObjects,
    model::*,
};
use super::definitions::{
    BufferComponent,
    create_push_constant_data,
    PushConstantData,
    TransformComponent,
    UniformBufferObject,
    VertexIndexComponent,
    Vertex2d,
    VertexBuffer,
};
use crate::adel_camera::Camera;
use crate::adel_tools::*;

use winit::window::Window;
use std::rc::{Rc};
const NAME: &'static str = "Renderer";

// May have to invert the order here, as the values of structs are dropped in the order they are declared
pub struct RendererAsh {
    // vulkan stuff
    _entry: ash::Entry,
    context: AshContext,
    pub device: ash::Device,

    swapchain: AshSwapchain,

    pipeline: AshPipeline,
    buffers: AshBuffers,

    sync_objects: SyncObjects,
    current_frame: usize,
    is_framebuffer_resized: bool,
    descriptor_set: Option<Vec<vk::DescriptorSet>>,

    pub window: Rc<Window>,

    name: &'static str,

    is_frame_started: bool,
}

impl RendererAsh {
    pub fn new(window: Rc<Window>) -> Result<Self> {
        // init vulkan stuff
        let entry = unsafe { ash::Entry::load()? };
        let context = AshContext::new(&entry, &window)?;
        let device = create_logical_device(&context, &VALIDATION_LAYERS.to_vec())?;
        let swapchain = AshSwapchain::new(&context, &device, &window)?;

        let pipeline = AshPipeline::new(&device, swapchain.format(), AshBuffers::get_depth_format(&context)?, swapchain.extent())?;
        let buffers = AshBuffers::new(&device, &context, &swapchain, &pipeline)?;

        let sync_objects = SyncObjects::new(&device, MAX_FRAMES_IN_FLIGHT)?;

        Ok(Self {
            _entry: entry,
            context,
            device,
            swapchain,
            pipeline,
            buffers,
            sync_objects,
            current_frame: 0,
            is_framebuffer_resized: false,
            descriptor_set: None,


            window,
            name: NAME,
            is_frame_started: false,
        })

    }
    // Will be worth revisiting at a later time if splitting up draw_frame is desired
    fn begin_frame(&mut self) -> Result<([vk::Fence; 1], u32, vk::CommandBuffer)> {
        // Wait for the fences to clear prior to beginning the next render
        let wait_fences = [self.sync_objects.inflight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)?;
        }

        // May need to find out where to put this
        //assert_eq!(self.is_frame_started, false);
        // Acquire the next image in the swapchain
        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain.swapchain_loader().acquire_next_image(
                self.swapchain.swapchain(),
                std::u64::MAX,
                self.sync_objects.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) =>
                    match vk_result {
                        // TODO: If getting the image index fails crash the program
                        //vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        //    self.recreate_swapchain();
                        //    return;
                        //}
                        _ => return Err(anyhow!("Failed to acquire Swap Chain Image!")),
                },
            }
        };
        self.is_frame_started = true;
        // Set isFrameStarted to true
        // Get the command buffer from the vector that is equal to the current frame
        let command_buffer = self.buffers.command_buffers()[self.current_frame];
        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.device.
                begin_command_buffer(command_buffer, &begin_info)?;
        }
        Ok( (wait_fences, image_index, command_buffer) )
    }
    fn begin_swapchain_render_pass(&mut self, image_index: u32, command_buffer: &vk::CommandBuffer) {
        // BeginSwapcahinRenderPass
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        // Messy but it functions
        let render_area = vk::Rect2D {
            offset: vk::Offset2D::builder().x(0).y(0).build(),
            extent: self.swapchain.extent()
        };
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.pipeline.render_pass().clone())
            .render_area(render_area)
            .framebuffer(self.buffers.framebuffers()[image_index as usize])
            .clear_values(&clear_values)
            .build();

        let viewport = [
            vk::Viewport::builder()
                .x(0.0).y(0.0)
                .width(self.swapchain.extent().width as f32)
                .height(self.swapchain.extent().height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
            .build()
        ];
        let scissor = [vk::Rect2D {
            offset: vk::Offset2D::builder().x(0).y(0).build(),
            extent: self.swapchain.extent()
        }];
        unsafe {
            self.device
                .cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);

            self.device
                .cmd_set_viewport(*command_buffer, 0, &viewport);

            self.device
                .cmd_set_scissor(*command_buffer, 0, &scissor);
        }
    }
    fn end_swapchain_render_pass(&mut self, command_buffer: &vk::CommandBuffer) {
        unsafe {
            self.device
                .cmd_end_render_pass(*command_buffer);
        }
    }
    // No need for references here as the resources are consumed at the end of the frame, and new ones will be generated next frame
    fn end_frame(&mut self, image_index: u32, wait_fences: [vk::Fence; 1], command_buffer: vk::CommandBuffer) -> Result<()> {
        unsafe {
            self.device
                .end_command_buffer(command_buffer)?;
        }

        let wait_semaphores = [self.sync_objects.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync_objects.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[self.buffers.command_buffers[self.current_frame]])
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device
                .reset_fences(&wait_fences)?;

            self.device
                .queue_submit(
                    self.swapchain.graphics_queue,
                    &submit_infos,
                    self.sync_objects.inflight_fences[self.current_frame],
                )?;
        }

        let swapchains = [self.swapchain.swapchain()];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&[image_index])
            .build();

        let result = unsafe {
            self.swapchain.swapchain_loader()
                .queue_present(self.swapchain.present_queue, &present_info)
        };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => return Err(anyhow!("Failed to execute queue present.")),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }
        // isFrameStarted = true;
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
    // TODO: Break up this function
    //pub fn draw_frame(&mut self, buffers: Vec<(&BufferComponent, PushConstantData)>) {
    //pub fn draw_frame(&mut self, buffers: Vec<&BufferComponent>) {
    pub fn draw_frame(&mut self, models: Vec<&ModelComponent>, model_matrix: nalgebra::Matrix4::<f32>, proj: nalgebra::Matrix4::<f32>, view: nalgebra::Matrix4::<f32>
        ) -> Result<()>
    {
        // Begin_frame requires a return of Image_index, wait_fences and command_buffer
        let (wait_fences, image_index, command_buffer) = self.begin_frame()?;
        self.begin_swapchain_render_pass(image_index, &command_buffer);
        // Render Objects
        //bind pipeline
        let device_size_offsets: [vk::DeviceSize; 1] = [0];
        //let descriptor_sets = self.buffers.descriptor_sets.as_ref().unwrap();
        unsafe {
            self.device
                .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline());
            for model in models.iter() {
                AshBuffers::update_uniform_buffer_new(&self.device, &model.uniform_buffers_memory, self.current_frame, model_matrix, proj, view);
                let descriptor_sets_to_bind = [model.descriptor_sets[self.current_frame]];
                self.device
                    .cmd_bind_vertex_buffers(command_buffer,
                        0,
                        &[model.vertex_buffer],
                        &device_size_offsets
                    );
                self.device.cmd_bind_index_buffer(command_buffer, model.index_buffer, 0, vk::IndexType::UINT32);
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.pipeline_layout(),
                    0,
                    &descriptor_sets_to_bind,
                    &[]
                );
                //self.device
                //    .cmd_push_constants(command_buffer,
                //        self.pipeline.pipeline_layout(),
                //        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                //        0,
                //        as_bytes(&buffer.1)
                //    );

                self.device.cmd_draw_indexed(command_buffer, model.indices_count, 1, 0, 0, 0);
            }
        }

        self.end_swapchain_render_pass(&command_buffer);
        self.end_frame(image_index, wait_fences, command_buffer)?;
        Ok(())
    }

    fn recreate_swapchain(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.destroy_swapchain_resources();
        let width_height = (self.window.inner_size().width, self.window.inner_size().height);
        self.context.surface_info.update_screen_width_height(width_height.0, width_height.1);
        self.swapchain.recreate_swapchain(
            &self.context,
            &self.device,
            &self.window,
        );
        self.buffers.recreate_depth_image(&self.context, &self.device, self.swapchain.swapchain_info.swapchain_extent);
        self.pipeline.recreate_render_pass(&self.device, self.swapchain.swapchain_info.swapchain_format);
        self.buffers.recreate_framebuffers(
            &self.device,
            self.pipeline.render_pass().clone(),
            &self.swapchain.image_views(),
            self.buffers.depth_image_view().clone(),
            self.swapchain.extent(),
        );
        // NOTE: sync_objects may need to be recreated if the total number of frames in flight changed
    }

    fn destroy_swapchain_resources(&mut self) {
        unsafe {
            self.swapchain.destroy_swapchain(&self.device);
            self.pipeline.destroy_render_pass(&self.device);
            self.buffers.destroy_framebuffers(&self.device);
            self.buffers.destroy_depth_image(&self.device);
        }
    }


}
pub fn create_push_constant_data_tmp(tmp : Matrix4<f32>) -> PushConstantData {
    PushConstantData {
        transform: tmp, //camera_projection,
        color: Vector3::new(0.0, 1.0, 0.0),
    }
}

use crate::adel_ecs::RunStage;
impl System for RendererAsh {
    fn startup(&mut self, world: &mut World) {
        let mut model_vec: Vec<Option<ModelComponent>> = Vec::new();
        //let mut buffer_component_vec: Vec<Option<BufferComponent>> = Vec::new();

        {
            let model_component_builder = world.borrow_component::<ModelComponentBuilder>().unwrap();
            for mc in model_component_builder.iter() {
                if let Some(component) = mc {
                    // TODO: Make Component Push fuction to account for None
                    model_vec.push(Some(component.build(&self.context, &self.device, &self.buffers, self.pipeline.descriptor_set_layout()).expect("Failed to build model")));
                } else {
                    model_vec.push(None);
                }
            }

            /*
            let vertex_index = world.borrow_component::<VertexIndexComponent>().unwrap();
            for vi in vertex_index.iter() {
                if let Some(component) = vi {
                    let (vertex_buffer, vertex_buffer_memory) = self.buffers.create_vertex_buffer(&self.context, &self.device, &component.vertices);
                    let (index_buffer, index_buffer_memory) = self.buffers.create_index_buffer(&self.context, &self.device, &component.indices);
                    let buffer_component = BufferComponent {
                        vertex_buffer,
                        vertex_buffer_memory,
                        index_buffer,
                        index_buffer_memory,
                        indices_count: component.indices.len() as u32,
                    };
                    buffer_component_vec.push(Some(buffer_component));
                }
                else {
                    buffer_component_vec.push(None);
                }
            }
            */
        }
        //world.insert_component(buffer_component_vec);
        world.insert_component(model_vec);
    }
    fn run(&mut self, world: &mut World) {
        //let option_buffers = world.borrow_component::<BufferComponent>().unwrap();
        let models = world.borrow_component::<ModelComponent>().unwrap();
        let mut transform_component = world.borrow_component_mut::<TransformComponent>().unwrap();
        let camera = world.get_resource::<Camera>().unwrap();
        //let mut buffers_push_constant: Vec<(&BufferComponent, PushConstantData)> = Vec::new();
        let mut model_vec: Vec<&ModelComponent> = Vec::new();
        let mut model_matrix = nalgebra::Matrix4::identity();
        for i in models.iter().enumerate() {
            if let Some(buffer) = i.1 {
                if let Some(transform) = &mut transform_component[i.0] {
                    transform.rotation.z += 0.25 * world.get_dt();
                    //transform.rotation.y -= (0.25 * world.get_dt());
                    model_matrix = transform.mat4_less_computation();
                    model_vec.push(buffer);
                }
            }
        }
        /*let mut buffers_push_constant: Vec<&BufferComponent> = Vec::new();
        for i in option_buffers.iter().enumerate() {
            if let Some(buffer) = i.1 {
                if let Some(transform) = &transform_component[i.0] {
                    buffers_push_constant.push(&buffer);
                }
            }
        }
        */
        self.draw_frame(model_vec, model_matrix, camera.get_projection(), camera.get_view());
    }
    // TODO: When Uniform buffers, Textures, and Models are abstracted to components, they need to be freed here
    fn shutdown(&mut self, world: &mut World) {
        unsafe {
            self.device.device_wait_idle().expect("ERROR: Failed to wait device idle on shutdown");

            if let Some(mut buffers) = world.borrow_component_mut::<BufferComponent>() {
                for i in buffers.iter_mut() {
                    if let Some(buffer) = i {
                        self.device.destroy_buffer(buffer.vertex_buffer, None);
                        self.device.free_memory(buffer.vertex_buffer_memory, None);
                        self.device.destroy_buffer(buffer.index_buffer, None);
                        self.device.free_memory(buffer.index_buffer_memory, None);
                    }
                }
            }
            if let Some(mut models) = world.borrow_component_mut::<ModelComponent>() {
                for model in models.iter_mut() {
                    if let Some(i) = model {
                        i.destroy_model_component(&self.device);
                    }
                }
            }
        };
    }
    fn name(&self) -> &'static str { self.name }

    // Update doesn't recreate swapchain
    fn get_run_stage(&self) -> RunStage {
        RunStage::RedrawUpdate
    }
}

impl Drop for RendererAsh {
    fn drop(&mut self) {
        unsafe {
            // Destroys Fences and Semaphores
            self.sync_objects.cleanup_sync_objects(&self.device, MAX_FRAMES_IN_FLIGHT);

            self.buffers.destroy_all(&self.device);

            // Destorys Swapchain and ImageViews
            self.swapchain.destroy_swapchain(&self.device);
            self.pipeline.destroy_descriptor_set_layout(&self.device);
            self.pipeline.destroy_render_pass(&self.device);

            // Destroys Pipeline and PipelineLayout
            self.pipeline.destroy_pipeline(&self.device);

            self.device.destroy_device(None);
            self.context.destroy_context();
        }
    }
}