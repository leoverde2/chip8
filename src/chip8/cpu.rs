use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::backend::backend::Backend;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Cpu{
    memory: [u8; 4096],
    framebuffer: [u8; WIDTH * HEIGHT],

    registers: Registers,
    pc: u16,
    i: u16,
    cycle_handler: CycleHandler,
}

#[allow(non_snake_case)]
#[derive(Default)]
pub struct Registers{
    V0: u8,
    V1: u8,
    V2: u8,
    V3: u8,
    V4: u8,
    V5: u8,
    V6: u8,
    V7: u8,
    V8: u8,
    V9: u8,
    VA: u8,
    VB: u8,
    VC: u8,
    VD: u8,
    VE: u8,
    VF: u8,
}

fn xy_to_1d(x: u8, y: u8) -> usize{
    let x = x as usize;
    let y = y as usize;

    x + y * WIDTH
}

impl Registers{
    pub fn get_register_by_nibble<'a>(&'a mut self, nibble: u8) -> &'a mut u8{
        match nibble{
            0x0 => &mut self.V0,
            0x1 => &mut self.V1,
            0x2 => &mut self.V2,
            0x3 => &mut self.V3,
            0x4 => &mut self.V4,
            0x5 => &mut self.V5,
            0x6 => &mut self.V6,
            0x7 => &mut self.V7,
            0x8 => &mut self.V8,
            0x9 => &mut self.V9,
            0xA => &mut self.VA,
            0xB => &mut self.VB,
            0xC => &mut self.VC,
            0xD => &mut self.VD,
            0xE => &mut self.VE,
            0xF => &mut self.VF,
            _ => panic!(),
        }
    }

    pub fn get_register_value(&mut self, nibble: u8) -> u8 {
        *self.get_register_by_nibble(nibble)
    }

    pub fn set_register_value(&mut self, nibble: u8, value: u8){
        let reg = self.get_register_by_nibble(nibble);
        *reg = value;
    }
}

pub struct CycleHandler{
    last_cycle: Instant,
    cycles_per_second: u32,
    cycle_duration: Duration,
}

pub struct Instruction{
    opcode: [u8; 2]
}

impl Instruction{
    pub fn get_nibble(&self, idx: usize) -> u8{
        assert!(idx < 4);
        let byte = self.opcode[idx / 2];
        let result = if idx % 2 == 0{
            (byte >> 4) & 0xF
        } else {
            byte & 0xF
        };
        result
    }

    pub fn get_address(&self) -> u16{
        let instruction = self.get_u16_instruction();
        instruction & 0xFFF
    }

    pub fn get_u16_instruction(&self) -> u16{
        ((self.opcode[0] as u16) << 8) | (self.opcode[1] as u16)
    }
}


impl<> Cpu{
    pub fn new() -> Self{
        let memory = [0; 4096];
        let framebuffer = [0; 64 * 32];
        let cycles_per_second = 700;
        let registers = Registers::default();

        let mut cpu = Self{
            registers,
            memory,
            i: 0,
            framebuffer,
            cycle_handler: CycleHandler{
                cycles_per_second,
                cycle_duration: Duration::from_secs_f64(1.0 / cycles_per_second as f64),
                last_cycle: Instant::now(),
            },
            pc: 0x200,
        };
        cpu.load_rom();
        cpu
    }
}

impl Cpu{
    pub fn load_rom(&mut self){
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("programs/IBM_Logo.ch8");
        let rom_data = fs::read(&path).expect("Failed to read ROM file");

        let start_address = 0x200;
        let end_address = start_address + rom_data.len();
        self.memory[start_address..end_address].copy_from_slice(&rom_data);
    }

    pub fn tick<B: Backend>(&mut self, backend: &mut B){
        let now = Instant::now();

        while now.duration_since(self.cycle_handler.last_cycle) >= self.cycle_handler.cycle_duration{
            self.fetch(backend);
            self.cycle_handler.last_cycle += self.cycle_handler.cycle_duration;
        }
    }

    fn fetch<B: Backend>(&mut self, backend: &mut B){
        let pc = self.pc as usize;
        let instruction = Instruction{opcode: self.memory[pc..pc + 2].try_into().unwrap()};
        self.pc += 2;
        self.decode(backend, instruction);
    }

    fn decode<B: Backend>(&mut self, backend: &mut B, instruction: Instruction){
        let first_nibble = instruction.get_nibble(0);

        match first_nibble{
            0x0 => self.framebuffer = [0; 64 * 32],
            0x1 => {
                let address = instruction.get_address();
                self.pc = address;
            },
            0x6 => {
                let reg_nibble = instruction.get_nibble(1);
                let register = self.registers.get_register_by_nibble(reg_nibble);
                *register = instruction.opcode[1];
            },
            0x7 => {
                let reg_nibble = instruction.get_nibble(1);
                let register = self.registers.get_register_by_nibble(reg_nibble);
                let value = instruction.opcode[1];
                *register += value;
            },
            0xA => {
                let address = instruction.get_address();
                self.i = address;
            }
            0xD => {
                self.registers.VF = 0;

                let x = *self.registers.get_register_by_nibble(instruction.get_nibble(1));
                let y = *self.registers.get_register_by_nibble(instruction.get_nibble(2));
                let sprite_height = instruction.get_nibble(3);
                for y_sprite_idx in (0..sprite_height).into_iter(){
                    let sprite_byte = self.memory[self.i as usize + y_sprite_idx as usize];
                    for x_sprite_idx in (0..8).into_iter(){
                        let x_pos = (x + x_sprite_idx) as usize % 64;
                        let y_pos = (y + y_sprite_idx) as usize % 32;
                        let pixel_idx = xy_to_1d(x_pos.try_into().unwrap(), y_pos.try_into().unwrap());

                        let sprite_pixel = (sprite_byte >> (7 - x_sprite_idx)) & 1;
                        let current_pixel = self.framebuffer[pixel_idx];
                        if sprite_pixel == 1 && current_pixel == 1{
                            self.registers.VF = 1;
                        }

                        self.framebuffer[pixel_idx] ^= sprite_pixel;
                    }
                }
                backend.draw_frame(&self.framebuffer);
            }
            _ => {
                panic!("Instruction {:?} not implemented", first_nibble)
            },
        }
    }

    fn execute<B: Backend>(&mut self, backend: &mut B, instruction: [u8; 2]){
        backend.draw_frame(&self.framebuffer);
    }
}
