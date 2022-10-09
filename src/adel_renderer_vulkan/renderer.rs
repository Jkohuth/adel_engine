
use ash::vk;
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use log;
use crate::adel_renderer_vulkan::utility::structures::Vertex;
use nalgebra::{Vector2, Vector3};
use nalgebra;

use crate::adel_ecs::{System, World};
const VERTICES_DATA: [Vertex; 3] = [
    Vertex {
        position: Vector2::new(0.0, -0.5),
        color: Vector3::new(1.0, 0.0, 0.0),
    },
    Vertex {
        position: Vector2::new(0.5, 0.5),
        color: Vector3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        position: Vector2::new(-0.5, 0.5),
        color: Vector3::new(0.0, 0.0, 1.0),
    },
];
// TODO: Create a prelude and add these to it
use crate::adel_renderer_vulkan::utility::{
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
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

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
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    sync_objects: SyncObjects,

    current_frame: usize,

    is_framebuffer_resized: bool,

    push_const: structures::PushConstantData,
    pub window: Window,

    name: &'static str,
}

impl RendererAsh {
    pub fn new(window: Window) -> Self {
        // init vulkan stuff
        let entry = unsafe { ash::Entry::load().expect("Error: Failed to create Ash Entry") };
        let context = AshContext::new(&entry, &window);
        let device = create_logical_device(&context, &VALIDATION_LAYERS.to_vec());
        let swapchain = AshSwapchain::new(&context, &device, &window);
        let pipeline = AshPipeline::new(&device, swapchain.format(), swapchain.extent());
        let buffers = AshBuffers::new(&device, &context, &swapchain, &pipeline);

        let mut vertices_data: Vec<Vertex> = Vec::new();
        for i in VERTICES_DATA {
            vertices_data.push(i);
        }
        /*let (vertex_buffer, vertex_buffer_memory) = buffers::create_vertex_buffer(&instance, &device, physical_device, &vertices_data);
        let push_const = structures::PushConstantData {
            transform: nalgebra::Matrix4::identity(),
            color: nalgebra::Vector3::new(1.0, 0.0, 0.0),
        };
        let command_buffers = buffers::create_command_buffers_(
            &device,
            command_pool,
            graphics_pipeline,
            &framebuffers,
            render_pass,
            swapchain_info.swapchain_extent,
            vertex_buffer,
            &push_const,
            pipeline_layout.clone()
        );
        */
        let (vertex_buffer, vertex_buffer_memory) = buffers::create_vertex_buffer(&context.instance(), &device, context.physical_device, &vertices_data);
        let push_const = structures::PushConstantData {
            transform: nalgebra::Matrix4::identity(),
            color: nalgebra::Vector3::new(1.0, 0.0, 0.0),
        };

        let sync_objects = SyncObjects::new(&device, MAX_FRAMES_IN_FLIGHT);

        Self {
            _entry: entry,
            context,
            device,
            swapchain,
            pipeline,
            buffers,
            vertex_buffer,
            vertex_buffer_memory,
            sync_objects,
            current_frame: 0,
            is_framebuffer_resized: false,

            push_const,
            window,
            name: NAME,
        }

    }

}

impl RendererAsh {
    pub fn draw_frame(&mut self) {
        /*let wait_fences = [self.in_flight_fences[self.current_frame]];

        unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain_info.swapchain_loader.acquire_next_image(
                self.swapchain_info.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
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

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&[self.command_buffers[image_index as usize]])
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain_info.swapchain];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&[image_index])
            .build();

        let result =unsafe {
            self.swapchain_info.swapchain_loader
                .queue_present(self.present_queue, &present_info)
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

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;*/
    }

    fn recreate_swapchain(&mut self) {
        // parameters -------------
        let surface_info = structures::SurfaceInfo {
            surface_loader: self.context.surface_info.surface_loader.clone(),
            surface: self.context.surface_info.surface,
            screen_width: self.window.inner_size().width,
            screen_height: self.window.inner_size().height,
        };
        // ------------------------

        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        // TODO: Create proper cleanup function
        self.destroy_swapchain_resources();

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
        // NOTE: sync_objects may need to be recreated if the total number of frames changed
    }

    fn destroy_swapchain_resources(&mut self) {
        unsafe {
            self.swapchain.destroy_swapchain(&self.device);
            self.pipeline.destroy_render_pass(&self.device);
            self.buffers.destroy_framebuffers(&self.device);
        }
    }

}

impl System for RendererAsh {
    fn startup(&mut self, world: &mut World) {}
    fn run(&mut self, world: &mut World) {}
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

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.device.destroy_device(None);
            self.context.destroy_context();
        }
    }
}
