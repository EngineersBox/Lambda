
#![feature(cursor_remaining)]
use std::io::Cursor;

mod map;
mod resource;
mod scene;

#[macro_use]
extern crate glium;
extern crate glm;
extern crate bit_set;
extern crate byteorder;
extern crate bitter;



use glium::{glutin, Surface};

fn render(display: &glium::Display) {
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 1.0, 1.0);
    target.finish().unwrap();
}

fn main() {
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
