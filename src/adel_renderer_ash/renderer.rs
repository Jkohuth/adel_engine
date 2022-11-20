
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
    create_push_constant_data,
    PushConstantData,
    TransformComponent,
    TriangleComponent,
    Vertex2d,
    VertexBuffer,
};
use crate::adel_camera::Camera;
use crate::adel_tools::*;

use crate::adel_winit::WinitWindow;
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
    pub window: WinitWindow,

    name: &'static str,

    is_frame_started: bool,
}

impl RendererAsh {
    pub fn new(window: WinitWindow) -> Self {
        // init vulkan stuff
        let entry = unsafe { ash::Entry::load().expect("Error: Failed to create Ash Entry") };
        let context = AshContext::new(&entry, window.window_ref().unwrap());
        let device = create_logical_device(&context, &VALIDATION_LAYERS.to_vec());
        let swapchain = AshSwapchain::new(&context, &device, window.window_ref().unwrap());
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

}

impl RendererAsh {
    pub fn draw_frame(&mut self, vertex_buffers: Vec<(vk::Buffer, PushConstantData)>) {
        // Wait for the fences to clear prior to beginning the next render
        let wait_fences = [self.sync_objects.inflight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

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
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
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
                .cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);

            self.device
                .cmd_set_viewport(command_buffer, 0, &viewport);

            self.device
                .cmd_set_scissor(command_buffer, 0, &scissor);
        }

        // Render Objects
        //bind pipeline
        let device_size_offsets: [vk::DeviceSize; 1] = [0];
        unsafe {
            self.device
                .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline());
            for buffer in vertex_buffers {
                self.device
                    .cmd_bind_vertex_buffers(command_buffer,
                        0,
                        &[buffer.0],
                        &device_size_offsets
                    );
                self.device
                    .cmd_push_constants(command_buffer,
                        self.pipeline.pipeline_layout(),
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        as_bytes(&buffer.1)
                    );
                //self.device
                //    .cmd_push_constants(command_buffer,
                //        self.pipeline.pipeline_layout(),
                //        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                //        0,
                //        as_bytes(&self.push_const)
                //    );

                // TODO: Vertex count shouldn't be hardcoded but lets get this bread
                self.device
                    .cmd_draw(command_buffer, 3, 1, 0, 0);

            }
        }

        // End SwapchainRenderPass

        unsafe {
            self.device
                .cmd_end_render_pass(command_buffer);
        }
        // End frame
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

    fn recreate_swapchain(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.destroy_swapchain_resources();
        let width_height = self.window.window_width_height();
        self.context.surface_info.update_screen_width_height(width_height.0, width_height.1);
        self.swapchain.recreate_swapchain(
            &self.context,
            &self.device,
            self.window.window_ref().unwrap(),
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

    // Will be worth revisiting at a later time if splitting up draw_frame is desired
    #[allow(dead_code)]
    fn begin_frame(&mut self) -> vk::CommandBuffer {
        assert_eq!(self.is_frame_started, false);
        // Wait for the fences to clear prior to beginning the next render
        let wait_fences = [self.sync_objects.inflight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

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
        command_buffer
    }

}
use nalgebra::Matrix4;
pub fn create_push_constant_data_proj(camera_projection : Matrix4<f32>) -> PushConstantData {
    PushConstantData {
        transform: camera_projection,
        color: Vector3::new(0.0, 0.0, 0.0),
    }
}

impl System for RendererAsh {
    fn startup(&mut self, world: &mut World) {
        let mut vert_buffer_component_vec: Vec<Option<VertexBuffer>> = Vec::new();

        {
            // Borrow the vertices that have been supplied to the world
            let triangles = world.borrow_component::<TriangleComponent>().unwrap();
            for triangle in triangles.iter().enumerate() {
                match triangle.1 {
                    Some(triangle_component) => {
                        // TODO: Rename these variables
                        let (vertex_buffer, vertex_buffer_memory) = buffers::AshBuffers::create_vertex_buffer(&self.context, &self.device, triangle_component);
                        let vertex_buf: VertexBuffer = VertexBuffer { buffer: vertex_buffer, memory: vertex_buffer_memory };
                        vert_buffer_component_vec.push(Some(vertex_buf));
                    }, None => {
                        vert_buffer_component_vec.push(None);
                    }
                }
            }
        }

        world.insert_component(vert_buffer_component_vec);
    }
    fn run(&mut self, world: &mut World) {
        // Copied from adel_renderer(_vulkano)
        // Get the camera projection
        // Apply the transformation matrix to it
        // iterate through all the transforms and apply the camera translation
        // draw frame
        let camera = world.get_resource::<Camera>().unwrap();
        let projection_matrix = camera.get_projection() * camera.get_view();
        //let mut model_ref = world.borrow_component_mut::<ModelComponent>().unwrap();
        let mut transform_ref = world.borrow_component_mut::<TransformComponent>().unwrap();
        let vertex_option_buffers = world.borrow_component::<VertexBuffer>().unwrap();
        let mut buffers_push_constant: Vec<(vk::Buffer, PushConstantData)> = Vec::new();
        for i in vertex_option_buffers.iter().enumerate() {
            if let Some(buffer) = i.1 {
                if let Some(transform) = &mut transform_ref[i.0] {
                    buffers_push_constant.push((buffer.buffer.clone(),
                        //create_push_constant_data(projection_matrix, transform)));
                        create_push_constant_data_proj(projection_matrix)));
                }
            }
        }
        self.draw_frame(buffers_push_constant);
    }
    fn shutdown(&mut self, world: &mut World) {
        unsafe {
            self.device.device_wait_idle().expect("ERROR: Failed to wait device idle on shutdown");

            let mut vertex_buffers = world.borrow_component_mut::<VertexBuffer>().unwrap();

            for i in vertex_buffers.iter_mut() {
                match i {
                    Some(buffer) => {
                        self.device.destroy_buffer(buffer.buffer, None);
                        self.device.free_memory(buffer.memory, None);
                    }, None => {}
                }
            }

        };
    }
    fn name(&self) -> &'static str { self.name }
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
            self.buffers.destroy_command_pool(&self.device);

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
