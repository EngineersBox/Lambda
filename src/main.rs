
mod map;
mod resource;
mod scene;
mod logging;
mod util;
mod rendering;
mod core;
mod input;

extern crate glium;
extern crate glm;
extern crate bit_set;
extern crate byteorder;
extern crate bitter;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;
extern crate slog_json;
extern crate lazy_static;
extern crate arr_macro;
extern crate std_tools;
extern crate imgui;
extern crate imgui_glium_renderer;
extern crate image;

use std::panic;

use glium::{glutin, Surface};
pub(crate) use lazy_static::lazy_static;
use slog::Logger;

use crate::logging::logging::initialize_logging;

lazy_static! {
    static ref LOGGER: Logger = initialize_logging(String::from("Lambda"));
}

fn render(display: &glium::Display) {
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 1.0, 1.0);
    target.finish().unwrap();
}

fn original_main() {
    info!(&crate::LOGGER, "Configured logging");
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new();
    let context_builder = glutin::ContextBuilder::new();
    let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();
    
    event_loop.run(move |ev, _, control_flow| {

        render(&display);

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }
    });
}

fn main() {
    info!(&crate::LOGGER, "Configured Logging");
    // NOTE: Temporary debugging panic logger
    panic::set_hook(Box::new(|panic_info: &panic::PanicInfo| {
        if let Some(location) = panic_info.location() {
            if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                crit!(
                    &crate::LOGGER,
                    "[{}:{}:{}] Panic with payload: {:?}",
                    location.file(),
                    location.line(),
                    location.column(),
                    msg,
                );
                std::thread::sleep(std::time::Duration::from_millis(1000));
                return;
            }

            crit!(
                &crate::LOGGER,
                "[{}:{}:{}] Panic with message: {}",
                location.file(),
                location.line(),
                location.column(),
                panic_info.to_string(),
            );
            std::thread::sleep(std::time::Duration::from_millis(1000));
            return
        }
        crit!(&crate::LOGGER, "Panic at unknown location");
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }));
    let bsp = map::bsp::BSP::from_file(&"maps/test3.bsp".to_string()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000));

}
