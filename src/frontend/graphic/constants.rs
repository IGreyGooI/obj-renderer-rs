pub const WINDOW_SIZE: winit::dpi::LogicalSize = winit::dpi::LogicalSize {
    width: RENDER_SIZE.width as f64,
    height: RENDER_SIZE.height as f64,
};
pub const RENDER_SIZE: gfx_hal::window::Extent2D = gfx_hal::window::Extent2D {
    width: 1920,
    height: 1080,
};
pub const WINDOW_TITLE: &str = &"gem";
pub const INSTANCE_NAME: &str = WINDOW_TITLE;