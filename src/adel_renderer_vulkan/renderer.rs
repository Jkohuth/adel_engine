
use ash::vk;
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use log;
use crate::adel_renderer_vulkan::utility::structures::Vertex;
use nalgebra::{Vector2, Vector3};
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
    debug,
    platforms,
    structures,
    tools,
    swapchain,
    device,
    pipeline,
    buffers,
};
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};

pub struct VulkanApp {
    // vulkan stuff
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_info: structures::SurfaceInfo,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    _physical_device: vk::PhysicalDevice,
    device: ash::Device,

    queue_family: structures::QueueFamilyIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_info: structures::SwapChainInfo,
    swapchain_imageviews: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,
    graphics_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,

    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    window: winit::window::Window,

    is_framebuffer_resized: bool,

    push_const: structures::PushConstantData,
}

impl VulkanApp {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_title("Test Window")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(event_loop)
            .expect("Failed: Create window");

        // init vulkan stuff
        let entry = unsafe { ash::Entry::load().expect("Error: Failed to create Ash Entry") };
        let instance = device::create_instance(&entry, ENABLE_VALIDATION_LAYERS, &VALIDATION_LAYERS.to_vec());
        let (debug_utils_loader, debug_messenger) = debug::setup_debug_utils(ENABLE_VALIDATION_LAYERS, &entry, &instance);
        let surface_info = device::create_surface(&entry, &instance, &window);
        let physical_device = device::pick_physical_device(&instance, &surface_info);
        let (device, queue_family) = device::create_logical_device(&instance, physical_device, &surface_info, &VALIDATION_LAYERS.to_vec());
        let graphics_queue =
            unsafe { device.get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(queue_family.present_family.unwrap(), 0) };

        let swapchain_info = swapchain::create_swapchain(
                                &instance,
                                &device,
                                physical_device,
                                &window,
                                &surface_info,
                                &queue_family);
        let swapchain_imageviews = pipeline::create_image_views(&device, swapchain_info.swapchain_format, &swapchain_info.swapchain_images);

        let render_pass = pipeline::create_render_pass(&device, swapchain_info.swapchain_format);

        let (graphics_pipeline, pipeline_layout) = pipeline::create_graphics_pipeline(&device, render_pass.clone(), swapchain_info.swapchain_extent);

        let framebuffers = buffers::create_framebuffers(&device, render_pass.clone(), &swapchain_imageviews, swapchain_info.swapchain_extent);

        let command_pool = buffers::create_command_pool(&device, &queue_family);
        let mut vertices_data: Vec<Vertex> = Vec::new();
        for i in VERTICES_DATA {
            vertices_data.push(i);
        }
        let (vertex_buffer, vertex_buffer_memory) = buffers::create_vertex_buffer(&instance, &device, physical_device, &vertices_data);
        use nalgebra;
        let push_const = structures::PushConstantData {
            transform: nalgebra::Matrix4::identity(),
            color: nalgebra::Vector3::new(1.0, 0.0, 0.0),
        };
        log::info!("JAKOB push_const {:?}", &push_const);
        let command_buffers = buffers::create_command_buffers(
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
        let sync_objects = buffers::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        Self {
            _entry: entry,
            instance,
            surface_info,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
            device,
            queue_family,
            graphics_queue,
            present_queue,
            swapchain_info,
            swapchain_imageviews,
            render_pass,
            graphics_pipeline,
            pipeline_layout,
            framebuffers,
            command_pool,
            command_buffers,
            vertex_buffer,
            vertex_buffer_memory,
            image_available_semaphores: sync_objects.image_available_semaphores,
            render_finished_semaphores: sync_objects.render_finished_semaphores,
            in_flight_fences: sync_objects.inflight_fences,
            current_frame: 0,
            window,
            is_framebuffer_resized: false,

            push_const
        }

    }

}

impl VulkanApp {
    fn draw_frame(&mut self) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

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

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn recreate_swapchain(&mut self) {
        // parameters -------------
        let surface_info = structures::SurfaceInfo {
            surface_loader: self.surface_info.surface_loader.clone(),
            surface: self.surface_info.surface,
            screen_width: self.window.inner_size().width,
            screen_height: self.window.inner_size().height,
        };
        // ------------------------

        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.cleanup_swapchain();

        let swapchain_info = swapchain::create_swapchain(
            &self.instance,
            &self.device,
            self._physical_device,
            &self.window,
            &surface_info,
            &self.queue_family,
        );
        self.swapchain_info.swapchain_loader = swapchain_info.swapchain_loader;
        self.swapchain_info.swapchain = swapchain_info.swapchain;
        self.swapchain_info.swapchain_images = swapchain_info.swapchain_images;
        self.swapchain_info.swapchain_format = swapchain_info.swapchain_format;
        self.swapchain_info.swapchain_extent = swapchain_info.swapchain_extent;

        self.swapchain_imageviews = pipeline::create_image_views(
            &self.device,
            self.swapchain_info.swapchain_format,
            &self.swapchain_info.swapchain_images,
        );
        self.render_pass = pipeline::create_render_pass(&self.device, self.swapchain_info.swapchain_format);

        self.framebuffers = buffers::create_framebuffers(
            &self.device,
            self.render_pass,
            &self.swapchain_imageviews,
            self.swapchain_info.swapchain_extent,
        );
        self.command_buffers = buffers::create_command_buffers(
            &self.device,
            self.command_pool,
            self.graphics_pipeline.clone(),
            &self.framebuffers,
            self.render_pass,
            self.swapchain_info.swapchain_extent,
            self.vertex_buffer,
            &self.push_const,
            self.pipeline_layout.clone(),
        );
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_info.swapchain_loader
                .destroy_swapchain(self.swapchain_info.swapchain, None);
        }
    }

    pub fn main_loop(mut self, event_loop: EventLoop<()>) {

        event_loop.run(move |event, _, control_flow| {

            match event {
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                | KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        | _ => {},
                                    }
                                },
                            }
                        },
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => {
                    self.window.request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    self.draw_frame();
                },
                | Event::LoopDestroyed => {
                    unsafe {
                        self.device.device_wait_idle()
                            .expect("Failed to wait device idle!")
                    };
                },
                _ => (),
            }

        })
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.cleanup_swapchain();
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.device.destroy_command_pool(self.command_pool, None);

            self.device.destroy_device(None);
            self.surface_info.surface_loader.destroy_surface(self.surface_info.surface, None);

            if ENABLE_VALIDATION_LAYERS {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}