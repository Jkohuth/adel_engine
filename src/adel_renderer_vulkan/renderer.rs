
use ash::vk;
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use log;
use std::ffi::{CString};
use crate::adel_renderer_vulkan::utility::structures::Vertex;
const VERTICES_DATA: [Vertex; 3] = [
    Vertex {
        position: [0.0, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        color: [0.0, 0.0, 1.0],
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
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    _physical_device: vk::PhysicalDevice,
    device: ash::Device,

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
        let (device, family_indices) = device::create_logical_device(&instance, physical_device, &surface_info, &VALIDATION_LAYERS.to_vec());
        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        let swapchain_info = swapchain::create_swapchain(
                                &instance,
                                &device,
                                physical_device,
                                &window,
                                &surface_info,
                                &family_indices);
        let swapchain_imageviews = pipeline::create_image_views(&device, swapchain_info.swapchain_format, &swapchain_info.swapchain_images);

        let render_pass = pipeline::create_render_pass(&device, swapchain_info.swapchain_format);

        let (graphics_pipeline, pipeline_layout) = pipeline::create_graphics_pipeline(&device, render_pass.clone(), swapchain_info.swapchain_extent);

        let framebuffers = buffers::create_framebuffers(&device, render_pass.clone(), &swapchain_imageviews, swapchain_info.swapchain_extent);

        let command_pool = buffers::create_command_pool(&device, &family_indices);
        let mut vertices_data: Vec<Vertex> = Vec::new();
        for i in VERTICES_DATA {
            vertices_data.push(i);
        }
        let (vertex_buffer, vertex_buffer_memory) = buffers::create_vertex_buffer(&instance, &device, physical_device, &vertices_data);
        let command_buffers = buffers::create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &framebuffers,
            render_pass,
            swapchain_info.swapchain_extent,
            vertex_buffer,
            vertices_data.len() as u32,
        );
        let sync_objects = buffers::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

        Self {
            _entry: entry,
            instance,
            surface_loader: surface_info.surface_loader,
            surface: surface_info.surface,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
            device,
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
        }

    }

}

impl VulkanApp {
    fn draw_frame(&mut self) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            self.swapchain_info.swapchain_loader
                .acquire_next_image(
                    self.swapchain_info.swapchain,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
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

        unsafe {
            self.swapchain_info.swapchain_loader
                .queue_present(self.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
/*
    fn recreate_swapchain(&mut self) {
        // parameters -------------
        let surface_suff = structures::SurfaceInfo {
            surface_loader: self.surface_loader.clone(),
            surface: self.surface,
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

        let swapchain_stuff = swapchain::create_swapchain(
            &self.instance,
            &self.device,
            self._physical_device,
            &self.window,
            &surface_suff,
            &self.queue_family,
        );
        self.swapchain_loader = swapchain_stuff.swapchain_loader;
        self.swapchain = swapchain_stuff.swapchain;
        self.swapchain_images = swapchain_stuff.swapchain_images;
        self.swapchain_format = swapchain_stuff.swapchain_format;
        self.swapchain_extent = swapchain_stuff.swapchain_extent;

        self.swapchain_imageviews = share::v1::create_image_views(
            &self.device,
            self.swapchain_format,
            &self.swapchain_images,
        );
        self.render_pass = share::v1::create_render_pass(&self.device, self.swapchain_format);
        let (graphics_pipeline, pipeline_layout) = VulkanApp::create_graphics_pipeline(
            &self.device,
            self.render_pass,
            swapchain_stuff.swapchain_extent,
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        self.swapchain_framebuffers = share::v1::create_framebuffers(
            &self.device,
            self.render_pass,
            &self.swapchain_imageviews,
            self.swapchain_extent,
        );
        self.command_buffers = buffers::create_command_buffers(
            &self.device,
            self.command_pool,
            self.graphics_pipeline,
            &self.framebuffers,
            self.render_pass,
            self.swapchain_extent,
        );
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }*/
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

            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);

            self.swapchain_info.swapchain_loader
                .destroy_swapchain(self.swapchain_info.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);

            if ENABLE_VALIDATION_LAYERS {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}