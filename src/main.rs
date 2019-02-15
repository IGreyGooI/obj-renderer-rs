#![warn(rust_2018_idioms, rust_2018_compatibility)]
#![feature(test)]
#![feature(specialization)]
#![feature(ptr_internals)]
#![allow(dead_code, unused_extern_crates, unused_imports)]

#[macro_use]
extern crate gfx_hal;
extern crate gfx_backend_vulkan as backend;
extern crate glutin;
extern crate image;
extern crate winit;
extern crate specs;

pub mod frontend;
pub mod lib;
pub mod app;

fn main() {
    let window_state = frontend::graphic::window::WindowState::new();
    let mut renderer_state =
        frontend::graphic::renderer::RendererState::new(
            &window_state,
            frontend::graphic::constants::RENDER_SIZE
        );
    'main: loop {
        renderer_state.try_rebuild_swapchain(frontend::graphic::constants::RENDER_SIZE);
        renderer_state.paint_frame();
        
    }
}
