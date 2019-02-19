#![warn(rust_2018_idioms, rust_2018_compatibility)]
#![feature(test)]
#![feature(specialization)]
#![feature(ptr_internals)]
#![allow(dead_code, unused_extern_crates, unused_imports)]

#[macro_use]
extern crate gfx_hal;
extern crate gfx_backend_dx12 as backend;
extern crate glutin;
extern crate image;
extern crate winit;
extern crate specs;
extern crate obj;

pub mod frontend;
pub mod lib;
pub mod app;

fn main() {
    let mut window_state = frontend::graphic::window::WindowState::new();
    /*
    let mut renderer_state =
        frontend::graphic::renderer::RendererState::new(
            &window_state,
            frontend::graphic::constants::RENDER_SIZE,
        );
    
    let mut running = true;
    'main: loop {
        window_state.events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event, .. } = event {
                match event {
                    winit::WindowEvent::KeyboardInput {
                        input: winit::KeyboardInput {
                            virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => running = false,
                    winit::WindowEvent::CloseRequested => running = false,
                    winit::WindowEvent::Resized(dims) => {
                        renderer_state.rebuild_swapchain = true;
                    }
                    _ => (),
                }
            }
        });
        if !running {
            break 'main;
        }
        renderer_state.try_rebuild_swapchain(frontend::graphic::constants::RENDER_SIZE);
        renderer_state.paint_frame();
    }
    */
}
