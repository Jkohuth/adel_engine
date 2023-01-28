use winit::{
    event_loop::EventLoop,
    window::{
        Window,
        WindowBuilder,
    },
};
use crate::adel_ecs::{System, World};
use std::rc::Rc;
const WINDOW_TITLE: &'static str = "Adel Engine";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct WinitWindow {
    window: Rc<Window>,
    event_loop: Option<EventLoop<()>>,
    name: &'static str
}
// TODO: Switching to ash means most of this window setup class can be rewritten
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
            window: Rc::new(window),
            event_loop: Some(event_loop),
            name: "WindowSystem"
        }
    }
    pub fn window_width_height(&self) -> (u32, u32) {
        (self.window.as_ref().inner_size().width, self.window.as_ref().inner_size().height)
    }
    pub fn rc_clone_window(&self) -> Rc<Window> {
        Rc::clone(&self.window)
    }
    pub fn event_loop(&mut self) -> Option<EventLoop<()>> {
        self.event_loop.take()
    }
}

impl System for WinitWindow {
    fn startup(&mut self, _world: &mut World) {}
    fn run(&mut self, _world: &mut World) {
        self.window.as_ref().request_redraw();
    }
    fn shutdown(&mut self, _world: &mut World) {}
    fn name(&self) -> &'static str {
        self.name
    }
}