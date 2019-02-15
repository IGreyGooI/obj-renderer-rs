use winit::{
    WindowBuilder, Window, EventsLoop, dpi::LogicalSize, WindowEvent,
};
use super::constants::*;

pub struct WindowState {
    pub window: Window,
    pub events_loop: EventsLoop,
}

impl WindowState {
    pub fn new() -> WindowState {
        let events_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_dimensions(WINDOW_SIZE)
            .with_title(WINDOW_TITLE)
            .build(&events_loop)
            .unwrap();
        WindowState {
            window,
            events_loop,
        }
    }
}