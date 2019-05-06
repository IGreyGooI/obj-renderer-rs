#![warn(rust_2018_idioms, rust_2018_compatibility)]
#![feature(test)]
#![feature(specialization)]
#![feature(ptr_internals)]
#![allow(dead_code, unused_extern_crates, unused_imports)]
#![feature(naked_functions)]

extern crate gfs;
extern crate gfx_backend_vulkan as backend;
#[macro_use]
extern crate gfx_hal;
extern crate glutin;
extern crate image;
extern crate obj;
extern crate ron;
extern crate specs;
extern crate spirv_cross;
extern crate time;
extern crate winit;

pub use ron::ser::Serializer;
use specs::prelude::*;
use spirv_cross::{ErrorCode, glsl, spirv};
use time::Duration;

use crate::lib::math::camera::Camera;
use crate::lib::math::light::PointLight;
use crate::lib::util::HistoryDefault;

impl HistoryDefault for Duration {
    fn history_default() -> Duration {
        Duration::zero()
    }
}
pub mod frontend;
pub mod lib;
pub mod app;

const PI: f32 = ::std::f64::consts::PI as f32;
const FPS: f32 = 1000.0;

fn main() {
    let mut window_state = frontend::graphic::window::WindowState::new();
    
    let mut renderer_state =
        frontend::graphic::renderer::RendererState::new(
            &window_state,
            frontend::graphic::constants::RENDER_SIZE,
        );
    
    let mut fps_history: lib::util::History<Duration> = lib::util::History::new(32);
    let mut render_time_history: lib::util::History<Duration> = lib::util::History::new(32);
    let startup_time = time::now();
    let mut one_second_timer = time::now();
    let mut running = true;
    'main: loop {
        let loop_start = time::now();
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
    
        let duration = time::now() - startup_time;
    
        let angle = duration.num_milliseconds() as f32 / 1000.0 * 45.0;
        let light_angle = duration.num_milliseconds() as f32 / 1000.0 * 45.0;
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
                x: 4.0,
                y: -4.0,
                z: 4.0,
            }
        };
        let render_start = time::now();
        renderer_state.paint_frame(camera, light);
        render_time_history.push(time::now() - render_start);
        fps_history.push(time::now() - loop_start);
        {
            let duration = time::now() - one_second_timer;
            if duration > Duration::seconds(1) {
                let mut sum_loop_time = Duration::zero();
                let mut sum_render_time = Duration::zero();
                for duration in fps_history.into_iter() {
                    sum_loop_time = sum_loop_time + duration;
                }
                for duration in render_time_history.into_iter() {
                    sum_render_time = sum_render_time + duration;
                }
                let average_loop_time = (sum_loop_time.num_nanoseconds().unwrap() as f64) /
                    (fps_history.count as f64);
                let average_render_time = (sum_render_time.num_nanoseconds().unwrap() as f64) /
                    (render_time_history.count as f64);
                let average_fps = 1_000_000_000.0 / average_loop_time;
                println!("FPS: {}", average_fps);
                println!("LT: {}", average_loop_time);
                println!("RT: {}", average_render_time);
                one_second_timer = time::now();
            }
        }
        //while (time::now() - frame_start_time) <
        //    Duration::milliseconds((1000.0 * (1.0 / FPS)) as i64) {}
    }
}





