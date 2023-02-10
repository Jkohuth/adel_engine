use anyhow::{anyhow, Result};
use ash::vk;
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use nalgebra::{Matrix4, Vector3};

use crate::adel_ecs::{System, World};
// TODO: Create a prelude and add these to it
use super::definitions::{BufferComponent, PushConstantData, TransformComponent};
use crate::adel_camera::Camera;
use crate::adel_renderer::utility::{
    constants::*,
    context::{create_logical_device, AshContext},
    descriptors::AshDescriptors,
    model::*,
    pipeline::AshPipeline,
    presenter::AshPresenter,
    swapchain::AshSwapchain,
    sync::SyncObjects,
};

use std::rc::Rc;
use std::sync::mpsc;
use winit::window::Window;
pub const NAME: &'static str = "Renderer";

// May have to invert the order here, as the values of structs are dropped in the order they are declared
pub struct RendererAsh {
    // vulkan stuff
    _entry: ash::Entry,
    context: AshContext,
    pub device: ash::Device,

    swapchain: AshSwapchain,

    pipeline: AshPipeline,
    presenter: AshPresenter,
    descriptors: AshDescriptors,

    sync_objects: SyncObjects,
    current_frame: usize,
    is_framebuffer_resized: bool,
    window_size: (u32, u32),
    //pub window: Rc<Window>,
    name: &'static str,
    receiver: mpsc::Receiver<(u32, u32)>,

    is_frame_started: bool,
}

impl RendererAsh {
    //pub fn new(window: Rc<Window>) -> Result<Self> {
    pub fn new(window: &Window, receiver: mpsc::Receiver<(u32, u32)>) -> Result<Self> {
        // init vulkan stuff
        let entry = unsafe { ash::Entry::load()? };
        let context = AshContext::new(&entry, &window)?;
        let device = create_logical_device(&context, &VALIDATION_LAYERS.to_vec())?;
        let window_size = (window.inner_size().width, window.inner_size().height);
        let swapchain = AshSwapchain::new(&context, &device, window_size)?;

        let pipeline = AshPipeline::new(
            &device,
            swapchain.format(),
            AshPresenter::get_depth_format(&context)?,
            swapchain.extent(),
        )?;
        let presenter = AshPresenter::new(&device, &context, &swapchain, &pipeline)?;
        let descriptors = AshDescriptors::new(&device, pipeline.descriptor_set_layout())?;

        let sync_objects = SyncObjects::new(&device, MAX_FRAMES_IN_FLIGHT)?;

        Ok(Self {
            _entry: entry,
            context,
            device,
            swapchain,
            pipeline,
            presenter,
            descriptors,
            sync_objects,
            current_frame: 0,
            is_framebuffer_resized: false,
            window_size,
            name: NAME,
            receiver,
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
                Err(vk_result) => match vk_result {
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
        let command_buffer = self.presenter.command_buffers()[self.current_frame];
        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)?;
        }
        Ok((wait_fences, image_index, command_buffer))
    }
    fn begin_swapchain_render_pass(
        &mut self,
        image_index: u32,
        command_buffer: &vk::CommandBuffer,
    ) {
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
            extent: self.swapchain.extent(),
        };
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.pipeline.render_pass().clone())
            .render_area(render_area)
            .framebuffer(self.presenter.framebuffers()[image_index as usize])
            .clear_values(&clear_values)
            .build();

        let viewport = [vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.swapchain.extent().width as f32)
            .height(self.swapchain.extent().height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build()];
        let scissor = [vk::Rect2D {
            offset: vk::Offset2D::builder().x(0).y(0).build(),
            extent: self.swapchain.extent(),
        }];
        unsafe {
            self.device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.device.cmd_set_viewport(*command_buffer, 0, &viewport);

            self.device.cmd_set_scissor(*command_buffer, 0, &scissor);
        }
    }
    fn end_swapchain_render_pass(&mut self, command_buffer: &vk::CommandBuffer) {
        unsafe {
            self.device.cmd_end_render_pass(*command_buffer);
        }
    }
    // No need for references here as the resources are consumed at the end of the frame, and new ones will be generated next frame
    fn end_frame(
        &mut self,
        image_index: u32,
        wait_fences: [vk::Fence; 1],
        command_buffer: vk::CommandBuffer,
    ) -> Result<()> {
        unsafe {
            self.device.end_command_buffer(command_buffer)?;
        }

        let wait_semaphores = [self.sync_objects.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync_objects.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[self.presenter.command_buffers[self.current_frame]])
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device.reset_fences(&wait_fences)?;

            self.device.queue_submit(
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
            self.swapchain
                .swapchain_loader()
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
            self.recreate_swapchain()?;
        }
        // isFrameStarted = true;
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
    // TODO: Break up this function
    //pub fn draw_frame(&mut self, buffers: Vec<(&BufferComponent, PushConstantData)>) {
    //pub fn draw_frame(&mut self, buffers: Vec<&BufferComponent>) {
    pub fn draw_frame(
        &mut self,
        models: Vec<&ModelComponent>,
        model_matrix: nalgebra::Matrix4<f32>,
        proj: nalgebra::Matrix4<f32>,
        view: nalgebra::Matrix4<f32>,
    ) -> Result<()> {
        // Begin_frame requires a return of Image_index, wait_fences and command_buffer
        let (wait_fences, image_index, command_buffer) = self.begin_frame()?;
        self.begin_swapchain_render_pass(image_index, &command_buffer);
        // Render Objects
        //bind pipeline
        let device_size_offsets: [vk::DeviceSize; 1] = [0];
        //let descriptor_sets = self.buffers.descriptor_sets.as_ref().unwrap();
        unsafe {
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.graphics_pipeline(),
            );
            for model in models.iter() {
                AshPresenter::update_uniform_buffer_new(
                    &self.device,
                    &model.uniform_buffers_memory,
                    self.current_frame,
                    model_matrix,
                    proj,
                    view,
                )?;
                let descriptor_sets_to_bind = [model.descriptor_sets[self.current_frame]];
                self.device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &[model.vertex_buffer],
                    &device_size_offsets,
                );
                self.device.cmd_bind_index_buffer(
                    command_buffer,
                    model.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.pipeline_layout(),
                    0,
                    &descriptor_sets_to_bind,
                    &[],
                );
                //self.device
                //    .cmd_push_constants(command_buffer,
                //        self.pipeline.pipeline_layout(),
                //        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                //        0,
                //        as_bytes(&buffer.1)
                //    );

                self.device
                    .cmd_draw_indexed(command_buffer, model.indices_count, 1, 0, 0, 0);
            }
        }

        self.end_swapchain_render_pass(&command_buffer);
        self.end_frame(image_index, wait_fences, command_buffer)?;
        Ok(())
    }

    fn recreate_swapchain(&mut self) -> Result<()> {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.destroy_swapchain_resources();
        let width_height = (
            self.window_size.0,
            self.window_size.1, //self.window.inner_size().width,
                                //self.window.inner_size().height,
        );
        self.context
            .surface_info
            .update_screen_width_height(width_height.0, width_height.1);
        self.swapchain
            .recreate_swapchain(&self.context, &self.device, self.window_size)?;
        self.presenter.recreate_depth_image(
            &self.context,
            &self.device,
            self.swapchain.swapchain_info.swapchain_extent,
        )?;
        self.pipeline
            .recreate_render_pass(&self.device, self.swapchain.swapchain_info.swapchain_format)?;
        self.presenter.recreate_framebuffers(
            &self.device,
            self.pipeline.render_pass().clone(),
            &self.swapchain.image_views(),
            self.presenter.depth_image_view().clone(),
            self.swapchain.extent(),
        )?;
        Ok(())
        // NOTE: sync_objects may need to be recreated if the total number of frames in flight changed
    }
    pub fn update_window_size(&mut self, width: u32, height: u32) {
        self.window_size = (width, height);
    }
    fn consume_events(&mut self) {
        if let Ok(window_size) = self.receiver.try_recv() {
            self.update_window_size(window_size.0, window_size.1);
            self.recreate_swapchain()
                .expect("Failed to recreate_swapchain");
        }
    }
    fn destroy_swapchain_resources(&mut self) {
        unsafe {
            self.swapchain.destroy_swapchain(&self.device);
            self.pipeline.destroy_render_pass(&self.device);
            self.presenter.destroy_framebuffers(&self.device);
            self.presenter.destroy_depth_image(&self.device);
        }
    }
}
pub fn create_push_constant_data_tmp(tmp: Matrix4<f32>) -> PushConstantData {
    PushConstantData {
        transform: tmp, //camera_projection,
        color: Vector3::new(0.0, 1.0, 0.0),
    }
}

use crate::adel_ecs::RunStage;
impl System for RendererAsh {
    fn startup(&mut self, world: &mut World) {
        let mut model_vec: Vec<Option<ModelComponent>> = Vec::new();

        {
            let model_component_builder =
                world.borrow_component::<ModelComponentBuilder>().unwrap();
            for mc in model_component_builder.iter() {
                if let Some(component) = mc {
                    // TODO: Make Component Push fuction to account for None
                    model_vec.push(Some(
                        component
                            .build(
                                &self.context,
                                &self.device,
                                &self.presenter,
                                &self.descriptors,
                            )
                            .expect("Failed to build model"),
                    ));
                } else {
                    model_vec.push(None);
                }
            }
        }

        world.insert_component(model_vec);
    }
    fn run(&mut self, world: &mut World) {
        self.consume_events();

        let models = world.borrow_component::<ModelComponent>().unwrap();
        let mut transform_component = world.borrow_component_mut::<TransformComponent>().unwrap();
        let camera = world.get_resource::<Camera>().unwrap();
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
        self.draw_frame(
            model_vec,
            model_matrix,
            camera.get_projection(),
            camera.get_view(),
        )
        .expect("Failed to draw frame");
    }
    // TODO: When Uniform buffers, Textures, and Models are abstracted to components, they need to be freed here
    fn shutdown(&mut self, world: &mut World) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("ERROR: Failed to wait device idle on shutdown");

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
    fn name(&self) -> &'static str {
        self.name
    }

    // Update doesn't recreate swapchain
    fn get_run_stage(&self) -> RunStage {
        RunStage::RedrawUpdate
    }
}

impl Drop for RendererAsh {
    fn drop(&mut self) {
        unsafe {
            // Destroys Fences and Semaphores
            self.sync_objects
                .cleanup_sync_objects(&self.device, MAX_FRAMES_IN_FLIGHT);

            self.descriptors.destroy_descriptor_pool(&self.device);
            self.presenter.destroy_all(&self.device);

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
