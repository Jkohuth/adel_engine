use std::sync::Arc;
use std::collections::HashMap;
use vulkano::{
    device::{
        Device,
        Queue,
    },
    format::Format,
    image::{ 
        AttachmentImage,
        ImageAccess,
        ImageDimensions,
        ImageUsage, 
        ImageCreateFlags,
        ImageViewAbstract,
        StorageImage, 
        swapchain::SwapchainImage,
        view::ImageView 
    },
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain,
    swapchain::{ AcquireError, Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError },
    sync,
    sync::{ FlushError, GpuFuture},
};

use vulkano_win::create_surface_from_winit;
use winit::window::Window;

use crate::adel_renderer::{VulkanoContext, FinalImageView, DeviceImageView};


// TODO: I've cleaned up and slimmed down just about all the code in other classes, however
// this class is still a bit fat. I will probably migrate some code into a sub class just to avoid 
// having giant files
pub struct VulkanoWindow {
    surface: Arc<Surface<Window>>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    swapchain: Arc<Swapchain<Window>>,
    final_views: Vec<FinalImageView>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    // Image view that is to be rendered with our pipeline.
    // (bool refers to whether it should get resized with swapchain resize)
    image_views: HashMap<usize, (DeviceImageView, bool)>,
    recreate_swapchain: bool,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    viewport: Viewport,

    previous_frame_end: Option<Box<dyn GpuFuture>>,
    image_index: usize,
}

impl VulkanoWindow {
    pub fn new(
        vulkano_context: &VulkanoContext, 
        window: Window,
    ) -> Self {
        let surface = create_surface_from_winit(window, vulkano_context.instance().clone()).unwrap();
        
        //let present_queue = vulkano_context.graphics_queue().filter(|&p| {
        //    p.supports_surface(&surface).unwrap()
        //});


        let present_queue = vulkano_context.graphics_queue().clone();
        let (swapchain, images, final_views) = vulkano_context.create_swapchain(&surface);
        let render_pass = Self::create_render_pass(vulkano_context.device(), &swapchain);
        let (framebuffers, viewport) = window_size_dependent_setup(vulkano_context.device(), &final_views.as_slice(), &render_pass);

        let previous_frame_end = Some(sync::now(vulkano_context.device().clone()).boxed());

        VulkanoWindow {
            surface,
            graphics_queue: vulkano_context.graphics_queue().clone(),
            present_queue,
            swapchain,
            final_views,
            images,
            image_views: HashMap::default(),
            recreate_swapchain: false,
            render_pass,
            framebuffers,
            viewport,
            previous_frame_end,
            image_index: 0,
        }
    }

    pub fn swapchain(&self) -> &Arc<Swapchain<Window>> {
        &self.swapchain
    }

    pub fn surface(&self) -> Arc<Surface<Window>> {
        self.surface.clone()
    }

    pub fn graphics_queue(&self) -> Arc<Queue> {
        self.graphics_queue.clone()
    }
    pub fn present_queue(&self) -> Arc<Queue> {
        self.present_queue.clone()
    }

    pub fn aspect_ratio(&self) -> f32 {
        let dims = self.window_size();
        dims[0] as f32 / dims[1] as f32
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    pub fn render_pass(&self) -> &Arc<RenderPass> {
        &self.render_pass
    }

    pub fn images(&self) -> &Vec<Arc<SwapchainImage<Window>>> {
        &self.images
    }

    pub fn window_size(&self) -> [u32; 2] {
        let size = self.window().inner_size();
        [size.width, size.height]
    }
    
    pub fn resize(&mut self) {
        self.recreate_swapchain = true;
    }

    pub fn final_views(&self) -> &Vec<FinalImageView> {
        &self.final_views
    }

    pub fn final_image(&self) -> &FinalImageView {
        &self.final_views[self.image_index]
    }

    pub fn final_image_size(&self) -> [u32; 2] {
        self.final_views[0].image().dimensions().width_height()
    }
    pub fn image_index(&self) -> usize {
        self.image_index
    }
    
    pub fn viewport(&mut self) -> &mut Viewport {
        &mut self.viewport
    }
    pub fn get_framebuffer(&self) -> &Arc<Framebuffer>{
        &self.framebuffers[self.image_index()]
    }

    pub fn add_image_target(&mut self, key: usize, view_size: Option<[u32; 2]>, format: Format) {
        let size = if let Some(s) = view_size {
            s
        } else {
            self.final_image_size()
        };
        let image = create_device_image(self.graphics_queue(), size, format);
        self.image_views.insert(key, (image, view_size.is_none()));
    }
    
    pub fn set_recreate_swapchain(&mut self, recreate: bool) {
        self.recreate_swapchain = recreate;
    }

    pub fn get_image_target(&mut self, key: usize) -> DeviceImageView {
        self.image_views.get(&key).unwrap().clone().0
    }
    pub fn remove_image_target(&mut self, key: usize) {
        self.image_views.remove(&key);
    }

    fn create_render_pass(device: &Arc<Device>, swapchain: &Arc<Swapchain<Window>>) 
        -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap()
    }

    fn recreate_swapchain_and_views(&mut self, device: &Arc<Device>) {
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: self.window().inner_size().into(),
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(e @ SwapchainCreationError::ImageExtentNotSupported {.. }) => {
                log::warn!("{}", e);
                return;
            }
            Err(e) => panic!("Failed to recreate swapchain {:?}", e),
        };

        self.swapchain = new_swapchain;
        self.images = new_images.clone();
        self.final_views = new_images.into_iter().map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>();
        let resizable_views = self
            .image_views
            .iter()
            .filter(|(_, (_img, follow_swapchain))| *follow_swapchain)
            .map(|c| *c.0)
            .collect::<Vec<usize>>();
        for i in resizable_views {
            let format = self.get_image_target(i).format().unwrap();
            self.remove_image_target(i);
            self.add_image_target(i, None, format);
        }
        let (framebuffers, viewport) = window_size_dependent_setup(device, self.final_views(), self.render_pass());
        self.framebuffers = framebuffers;
        self.viewport = viewport;

        self.recreate_swapchain = false;

    }
    
    // Acquires next swapchain image and increments image index
    // This is the first to call in render orchestration.
    // Returns a gpu future representing the time after which the swapchain image has been acquired
    // and previous frame ended.
    // After this, execute command buffers and return future from them to `finish_frame`.
    pub fn start_frame(&mut self, device: &Arc<Device>) -> std::result::Result<Box<dyn GpuFuture>, AcquireError> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            self.recreate_swapchain_and_views(device);
        }

        // Acquire next image
        let (image_num, suboptimal, acquire_future) = 
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Err(AcquireError::OutOfDate);
                }
                Err(e) => panic!("Failed to acquire the next image {:?}", e),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }
        self.image_index = image_num;
        let future = self.previous_frame_end.take().unwrap().join(acquire_future);
        
        Ok(future.boxed())
    }

    pub fn finish_frame(&mut self, after_future: Box<dyn GpuFuture>) {
        let future = after_future.then_swapchain_present(
            self.graphics_queue(),
            self.swapchain.clone(),
            self.image_index,
        )
        .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // A hack to prevent OutOfMemory error on Nvidia :(
                // https://github.com/vulkano-rs/vulkano/issues/627
                match future.wait(None) {
                    Ok(x) => x,
                    Err(err) => log::warn!("{:?}", err),
                }
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end =
                    Some(sync::now(self.graphics_queue.device().clone()).boxed());
            }
            Err(e) => {
                log::warn!("Failed to flush future: {:?}", e);
                self.previous_frame_end =
                    Some(sync::now(self.graphics_queue.device().clone()).boxed());
            }
        }
    }
}

pub fn window_size_dependent_setup(
    device: &Arc<Device>,
    images: &[FinalImageView],
    render_pass: &Arc<RenderPass>,
) -> (Vec<Arc<Framebuffer>>, Viewport) {
    let dimensions = images[0].image().dimensions().width_height();
    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(device.clone(), dimensions, Format::D16_UNORM).unwrap(),
    ).unwrap();
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    
    let framebuffers = images.iter().map(|image| {
        Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image.clone(), depth_buffer.clone()],
                ..Default::default()
            }
        ).unwrap()
    }).collect::<Vec<_>>();

    (framebuffers, viewport)
}

pub fn create_device_image(queue: Arc<Queue>, size: [u32; 2], format: Format) -> DeviceImageView {
    let dimensions = ImageDimensions::Dim2d {
        width: size[0],
        height: size[1],
        array_layers: 1,
    };
    let flags = ImageCreateFlags::none();
    ImageView::new_default(
        StorageImage::with_usage(
            queue.device().clone(),
            dimensions,
            format,
            ImageUsage {
                sampled: true,
                storage: true,
                color_attachment: true,
                transfer_dst: true,
                ..ImageUsage::none()
            },
            flags,
            Some(queue.family()),
        )
        .unwrap()
    )
    .unwrap()
}