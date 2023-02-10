use crate::adel_ecs::{System, World};
use std::rc::Rc;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
const WINDOW_TITLE: &'static str = "Adel Engine";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct WinitWindow {
    //window: Rc<Window>,
    window: Option<Window>,
    event_loop: Option<EventLoop<()>>,
    name: &'static str,
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
            ))
            .build(&event_loop)
            .unwrap();
        Self {
            window: Some(window),
            event_loop: Some(event_loop),
            name: "WindowSystem",
        }
    }
    pub fn window_width_height(window: &Window) -> (u32, u32) {
        (window.inner_size().width, window.inner_size().height)
    }
    //pub fn rc_clone_window(&self) -> Rc<Window> {
    //    Rc::clone(&self.window)
    //}
    pub fn window(&mut self) -> Option<Window> {
        self.window.take()
    }
    pub fn event_loop(&mut self) -> Option<EventLoop<()>> {
        self.event_loop.take()
    }
}

impl System for WinitWindow {
    fn startup(&mut self, _world: &mut World) {}
    fn run(&mut self, world: &mut World) {
        let window = world.get_resource::<Window>().unwrap();
        window.request_redraw();
    }
    fn shutdown(&mut self, _world: &mut World) {}
    fn name(&self) -> &'static str {
        self.name
    }
}
