
use ash::vk;
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use log;
use nalgebra::{Vector2, Vector3};
use nalgebra;

use crate::adel_ecs::{System, World};
// TODO: Create a prelude and add these to it
use crate::adel_renderer_ash::utility::{
    constants::*,
    platforms,
    structures,
    tools,
    swapchain::AshSwapchain,
    context::{AshContext, create_logical_device},
    pipeline::AshPipeline,
    buffers::AshBuffers,
    buffers,
    sync::SyncObjects,
};
use super::definitions::{
    BufferComponent,
    create_push_constant_data,
    PushConstantData,
    PushConstantData2D,
    TransformComponent,
    TriangleComponent,
    VertexIndexComponent,
    Vertex2d,
    VertexBuffer,
};
use crate::adel_camera::Camera;
use crate::adel_tools::*;

use crate::adel_winit::WinitWindow;
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

    push_const: PushConstantData,
    pub window: Rc<Window>,

    name: &'static str,

    is_frame_started: bool,
}

impl RendererAsh {
    pub fn new(window: Rc<Window>) -> Self {
        // init vulkan stuff
        let entry = unsafe { ash::Entry::load().expect("Error: Failed to create Ash Entry") };
        let context = AshContext::new(&entry, &window);
        let device = create_logical_device(&context, &VALIDATION_LAYERS.to_vec());
        let swapchain = AshSwapchain::new(&context, &device, &window);
        let pipeline = AshPipeline::new(&device, swapchain.format(), swapchain.extent());
        let buffers = AshBuffers::new(&device, &context, &swapchain, &pipeline);

        let push_const = PushConstantData {
            transform: nalgebra::Matrix4::identity(),
            color: nalgebra::Vector3::new(1.0, 0.0, 1.0),
        };

        let sync_objects = SyncObjects::new(&device, MAX_FRAMES_IN_FLIGHT);

        Self {
            _entry: entry,
            context,
            device,
            swapchain,
            pipeline,
            buffers,
            sync_objects,
            current_frame: 0,
            is_framebuffer_resized: false,

            push_const,
            window,
            name: NAME,
            is_frame_started: false,
        }

    }
    // Will be worth revisiting at a later time if splitting up draw_frame is desired
    fn begin_frame(&mut self) -> ([vk::Fence; 1], u32, vk::CommandBuffer) {
        // Wait for the fences to clear prior to beginning the next render
        let wait_fences = [self.sync_objects.inflight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
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
                        _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };
        self.is_frame_started = true;
        // Set isFrameStarted to true
        // Get the command buffer from the vector that is equal to the current frame
        let command_buffer = self.buffers.commandbuffers()[self.current_frame];
        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.device.
                begin_command_buffer(command_buffer, &begin_info)
                .expect("ERROR: Failed to begin command buffer");
        }
        (wait_fences, image_index, command_buffer)
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
    fn end_frame(&mut self, image_index: u32, wait_fences: [vk::Fence; 1], command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .expect("ERROR: Failed to end commandbuffer");
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
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.swapchain.graphics_queue,
                    &submit_infos,
                    self.sync_objects.inflight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
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
                _ => panic!("Failed to execute queue present."),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }
        // isFrameStarted = true;
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    // TODO: Break up this function
    pub fn draw_frame(&mut self, buffers: Vec<(&BufferComponent, PushConstantData)>) {
        // Begin_frame requires a return of Image_index, wait_fences and command_buffer
        let (wait_fences, image_index, command_buffer) = self.begin_frame();
        self.begin_swapchain_render_pass(image_index, &command_buffer);

        // Render Objects
        //bind pipeline
        let device_size_offsets: [vk::DeviceSize; 1] = [0];
        unsafe {
            self.device
                .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline());
            for buffer in buffers.iter() {
                self.device
                    .cmd_bind_vertex_buffers(command_buffer,
                        0,
                        &[buffer.0.vertex_buffer],
                        &device_size_offsets
                    );
                self.device.cmd_bind_index_buffer(command_buffer, buffer.0.index_buffer, 0, vk::IndexType::UINT16);
                self.device
                    .cmd_push_constants(command_buffer,
                        self.pipeline.pipeline_layout(),
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        as_bytes(&buffer.1)
                    );
                self.device.cmd_draw_indexed(command_buffer, buffer.0.indices_count, 1, 0, 0, 0);
            }
        }

        self.end_swapchain_render_pass(&command_buffer);
        self.end_frame(image_index, wait_fences, command_buffer);
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

        self.pipeline.recreate_render_pass(&self.device, self.swapchain.swapchain_info.swapchain_format);
        self.buffers.recreate_framebuffers(
            &self.device,
            self.pipeline.render_pass().clone(),
            &self.swapchain.image_views(),
            self.swapchain.extent(),
        );
        // NOTE: sync_objects may need to be recreated if the total number of frames in flight changed
    }

    fn destroy_swapchain_resources(&mut self) {
        unsafe {
            self.swapchain.destroy_swapchain(&self.device);
            self.pipeline.destroy_render_pass(&self.device);
            self.buffers.destroy_framebuffers(&self.device);
        }
    }


}
use nalgebra::{Matrix2, Matrix3, Matrix4};
use crate::adel_renderer_ash::Transform2dComponent;
pub fn create_push_constant_data_tmp(tmp : Matrix4<f32>) -> PushConstantData {
    PushConstantData {
        transform: tmp, //camera_projection,
        color: Vector3::new(0.0, 1.0, 0.0),
    }
}
pub fn create_push_constant_data_2d(transform_matrix: Matrix3<f32>) -> PushConstantData2D {
    PushConstantData2D {
        transform: transform_matrix, //camera_projection,
        //transform: Matrix3::identity(),
        color: Vector3::new(0.0, 1.0, 0.0),
    }
}

use crate::adel_ecs::RunStage;
impl System for RendererAsh {
    fn startup(&mut self, world: &mut World) {
        let mut buffer_component_vec: Vec<Option<BufferComponent>> = Vec::new();

        {
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
        }
        world.insert_component(buffer_component_vec);
    }
    fn run(&mut self, world: &mut World) {
        let option_buffers = world.borrow_component::<BufferComponent>().unwrap();
        let transform_component = world.borrow_component::<TransformComponent>().unwrap();
        let mut buffers_push_constant: Vec<(&BufferComponent, PushConstantData)> = Vec::new();
        for i in option_buffers.iter().enumerate() {
            if let Some(buffer) = i.1 {
                if let Some(transform) = &transform_component[i.0] {

                    buffers_push_constant.push((&buffer, create_push_constant_data_tmp(Matrix4::<f32>::identity())));
                    //buffers_push_constant.push((&buffer, create_push_constant_data_tmp(transform.mat4_less_computation())));

                }
            }
        }

        self.draw_frame(buffers_push_constant);
    }
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

            // Framebuffers, Commandbuffers, and CommandPool need cleanup
            // Framebuffers need to be separated as they are removed when recreating swapchain
            self.buffers.destroy_framebuffers(&self.device);
            self.buffers.free_command_buffers(&self.device);
            self.buffers.destroy_command_pools(&self.device);

            // Destorys Swapchain and ImageViews
            self.swapchain.destroy_swapchain(&self.device);
            self.pipeline.destroy_render_pass(&self.device);

            // Destroys Pipeline and PipelineLayout
            self.pipeline.destroy_pipeline(&self.device);

            self.device.destroy_device(None);
            self.context.destroy_context();
        }
    }
}
