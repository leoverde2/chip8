use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

use crate::backend::backend::{Backend, Keys};
use crate::chip8::cpu::Cpu;



pub enum PixelsBackend{
    Uninitialized,
    Initialized{
        inner: PixelsInner,
        cpu: Cpu
    },
}

pub struct PixelsInner{
    pub pixels: Pixels<'static>,
    pub window: Arc<Window>,
}

impl PixelsInner{
    pub fn new(window: Arc<Window>) -> Self{
        let surface_texture = SurfaceTexture::new(64, 32, window.clone());
        let mut pixels = Pixels::new(64, 32, surface_texture).unwrap();
        pixels.clear_color(pixels::wgpu::Color{r: 0.0, g: 0.0, b: 0.3, a: 1.0});

        Self{
            window: window.clone(),
            pixels,
        }
    }
}

impl Backend for PixelsInner{
    fn draw_frame(&mut self, framebuffer: &[u8; 64 * 32]) {
        let frame = self.pixels.frame_mut();
        for (i, &pixel) in framebuffer.iter().enumerate(){
            let rgba_idx = i * 4;
            let color = if pixel > 0 { 255 } else { 0 };
            frame[rgba_idx..rgba_idx + 4].copy_from_slice(&[color, color, color, 255]);
        }
        self.window.request_redraw();
    }

    fn poll_keys(&mut self) -> Vec<Keys> {
        todo!()
    }


}

impl ApplicationHandler for PixelsBackend{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self{
            PixelsBackend::Uninitialized => {
                let window = event_loop.create_window(WindowAttributes::default()).unwrap();
                let window = Arc::new(window);
                let inner = PixelsInner::new(window);
                let cpu = Cpu::new();
                *self = PixelsBackend::Initialized{inner, cpu};
            },
            PixelsBackend::Initialized{..} => return
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event{
            WindowEvent::Resized(size) => {
                match self {
                    PixelsBackend::Initialized{inner, cpu} => {
                        inner.pixels.resize_surface(size.width, size.height).unwrap();
                    },
                    PixelsBackend::Uninitialized => (),
                }
            },

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                match self {
                    PixelsBackend::Initialized{inner, cpu} => {
                        inner.pixels.render();
                    },
                    _ => (),
                }
            }


            _ => ()
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self {
            PixelsBackend::Uninitialized => (),
            PixelsBackend::Initialized{ref mut inner, ref mut cpu} => {
                cpu.tick(inner);
            }
        }
    }
}
