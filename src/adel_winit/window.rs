use winit::{
    event_loop::EventLoop,
    window::{
        Window,
        WindowBuilder,
    },
};

const WINDOW_TITLE: &'static str = "Adel Engine";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct WinitWindow {
    window: Option<Window>,
    event_loop: Option<EventLoop<()>>,
}

impl WinitWindow {
    pub fn new() -> Self {
        let event_loop: EventLoop<()> = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                WINDOW_WIDTH as f32,
                WINDOW_HEIGHT as f32,
            )).build(&event_loop).unwrap();
        Self {
            window: Some(window),
            event_loop: Some(event_loop),
        }
    }

    pub fn window(&mut self) -> Option<Window> {
        self.window.take()
    }
    
    pub fn event_loop(&mut self) -> Option<EventLoop<()>> {
        self.event_loop.take()
    }
}