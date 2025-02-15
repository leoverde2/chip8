use std::time::{Duration, Instant};

use crate::backend::backend::Backend;

pub struct Cpu{
    memory: [u8; 4096],
    framebuffer: [u8; 64 * 32],

    cycle_handler: CycleHandler,
}

pub struct CycleHandler{
    last_cycle: Instant,
    cycles_per_second: u32,
    cycle_duration: Duration,
}

impl<> Cpu{
    pub fn new() -> Self{
        let memory = [0; 4096];
        let framebuffer = [0; 64 * 32];
        let cycles_per_second = 700;

        let cpu = Self{
            memory,
            framebuffer,
            cycle_handler: CycleHandler{
                cycles_per_second,
                cycle_duration: Duration::from_secs_f64(1.0 / cycles_per_second as f64),
                last_cycle: Instant::now(),
            }
        };
        cpu
    }
}

impl Cpu{
    pub fn tick<B: Backend>(&mut self, backend: &mut B){
        let now = Instant::now();

        while now.duration_since(self.cycle_handler.last_cycle) >= self.cycle_handler.cycle_duration{
            self.execute_cycle(backend);
            self.cycle_handler.last_cycle += self.cycle_handler.cycle_duration;
        }
    }

    fn execute_cycle<B: Backend>(&mut self, backend: &mut B){
        backend.draw_frame(&self.framebuffer);
    }
}
