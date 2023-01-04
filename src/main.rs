
mod map;
mod resource;
mod scene;
mod logging;
mod util;

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

fn main() {
    info!(&crate::LOGGER, "Configured logging");
    let mut event_loop = glutin::event_loop::EventLoop::new();
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
