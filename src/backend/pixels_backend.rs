use std::collections::HashMap;
use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
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

pub enum WaitingKey{
    No,
    Yes,
}

pub struct PixelsInner{
    pub pixels: Pixels<'static>,
    pub window: Arc<Window>,
    pub waiting_key: WaitingKey,
    pub keys_pressed: HashMap<Keys, bool>,
}

impl PixelsInner{
    pub fn new(window: Arc<Window>) -> Self{
        let surface_texture = SurfaceTexture::new(64, 32, window.clone());
        let mut pixels = Pixels::new(64, 32, surface_texture).unwrap();
        pixels.clear_color(pixels::wgpu::Color{r: 0.0, g: 0.0, b: 0.3, a: 1.0});

        Self{
            window: window.clone(),
            pixels,
            waiting_key: WaitingKey::No,
            keys_pressed: HashMap::new(),
        }
    }

    pub fn keycode_to_keys(&self, code: KeyCode) -> Option<Keys>{
        match code{
            KeyCode::Digit1 => Some(Keys::KEY1),
            KeyCode::Digit2 => Some(Keys::KEY2),
            KeyCode::Digit3 => Some(Keys::KEY3),
            KeyCode::Digit4 => Some(Keys::KEY4),
            KeyCode::KeyQ => Some(Keys::KEYQ),
            KeyCode::KeyW => Some(Keys::KEYW),
            KeyCode::KeyE => Some(Keys::KEYE),
            KeyCode::KeyR => Some(Keys::KEYR),
            KeyCode::KeyA => Some(Keys::KEYA),
            KeyCode::KeyS => Some(Keys::KEYS),
            KeyCode::KeyD => Some(Keys::KEYD),
            KeyCode::KeyF => Some(Keys::KEYF),
            KeyCode::KeyZ => Some(Keys::KEYZ),
            KeyCode::KeyX => Some(Keys::KEYX),
            KeyCode::KeyC => Some(Keys::KEYC),
            KeyCode::KeyV => Some(Keys::KEYV),
            _ => None
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

    fn poll_key(&mut self, key: Keys) -> bool {
        if let Some(key) = self.keys_pressed.get(&key){
            *key
        } else {
            false
        }
    }

    fn wait_for_key(&mut self) { 
        self.waiting_key = WaitingKey::Yes;
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
                cpu.update_timers();
                match inner.waiting_key{
                    WaitingKey::No => cpu.tick(inner),
                    WaitingKey::Yes => return,
                }
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match self{
            Self::Uninitialized => return,
            Self::Initialized { ref mut inner, ref mut cpu } => {
                match event{
                    winit::event::DeviceEvent::Key(raw) => {
                        let state = raw.state;
                        let physical_key = raw.physical_key;
                        match physical_key {
                            PhysicalKey::Code(code) => {
                                let key = inner.keycode_to_keys(code);
                                let Some(key) = key else {return};
                                match state{
                                    ElementState::Released => {inner.keys_pressed.insert(key, false);},
                                    ElementState::Pressed => {
                                        inner.keys_pressed.insert(key, true);
                                        match inner.waiting_key{
                                            WaitingKey::Yes => {
                                                cpu.waiting_key_pressed(key);
                                                inner.waiting_key = WaitingKey::No;
                                            },
                                            WaitingKey::No => ()
                                        }
                                    },
                                };
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }


            }
        }
    }
}
