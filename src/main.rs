#![allow(unused)]

pub mod chip8;
pub mod backend;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowAttributes;

use crate::chip8::cpu::Cpu;
use crate::backend::pixels_backend::PixelsBackend;


fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut pixels_backend = PixelsBackend::Uninitialized;

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut pixels_backend);
}
