#![warn(rust_2018_idioms, rust_2018_compatibility)]
#![feature(test)]
#![feature(specialization)]
#![feature(ptr_internals)]
#![allow(dead_code, unused_extern_crates, unused_imports)]
#![feature(naked_functions)]

extern crate gfx_backend_dx12 as backend;
#[macro_use]
extern crate gfx_hal;
extern crate glsl_to_spirv;
extern crate glutin;
extern crate image;
extern crate obj;
extern crate specs;
extern crate time;
extern crate winit;

use time::Duration;

use crate::lib::math::camera::Camera;
use crate::lib::math::light::PointLight;

pub mod frontend;
pub mod lib;
pub mod app;

const PI: f32 = ::std::f64::consts::PI as f32;
const FPS: f32 = 12.0;

fn main() {
    let mut window_state = frontend::graphic::window::WindowState::new();
    
    let mut renderer_state =
        frontend::graphic::renderer::RendererState::new(
            &window_state,
            frontend::graphic::constants::RENDER_SIZE,
        );
    
    let startup_time = time::now();
    
    let mut running = true;
    'main: loop {
        let frame_start_time = time::now();
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
                        //renderer_state.rebuild_swapchain = true;
                    }
                    _ => (),
                }
            }
        });
        if !running {
            break 'main;
        }
        renderer_state.try_rebuild_swapchain(frontend::graphic::constants::RENDER_SIZE);
    
        let duration = time::now() - startup_time;
    
        let angle = duration.num_milliseconds() as f32 / 1000.0 * 45.0;
    
        let camera = Camera::perspective(
            cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            cgmath::Point3 {
                x: 2.0 * f32::sin(2.0 * PI * angle / 360.0),
                y: -2.0 * f32::sin(2.0 * PI * angle / 360.0),
                z: 2.0 * 1.414 * f32::cos(2.0 * PI * angle / 360.0),
            },
        );
        let light = PointLight {
            position: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 5.0,
            }
        };
    
        renderer_state.paint_frame(camera, light);
    
        while (time::now() - frame_start_time) < Duration::milliseconds((1000.0 * (1.0 / FPS))
            as i64) {}
    }
}

