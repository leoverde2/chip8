use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use rand::Rng;

use crate::backend::backend::{Backend, Keys};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const HEX_SPRITE_LEN: u16 = 10;
const HEX_SPRITE_START: u16 = 0x50;
const HEX_SPRITES: [u8; 80] = [
0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
0x20, 0x60, 0x20, 0x20, 0x70, // 1
0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
0x90, 0x90, 0xF0, 0x10, 0x10, // 4
0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
0xF0, 0x10, 0x20, 0x40, 0x40, // 7
0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
0xF0, 0x90, 0xF0, 0x90, 0x90, // A
0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
0xF0, 0x80, 0x80, 0x80, 0xF0, // C
0xE0, 0x90, 0x90, 0x90, 0xE0, // D
0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Cpu{
    memory: [u8; 4096],
    framebuffer: [u8; WIDTH * HEIGHT],

    registers: Registers,
    pc: u16,
    i: u16,
    dt: u8,
    st: u8,

    register_to_save_key: Option<u8>,

    stack: [u16; 16],
    sp: usize,
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
    last_tick: Instant,
    tick_duration: Duration,

    last_timer_update: Instant,
    timer_update_duration: Duration,
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
        let ticks_per_second = 700;
        let timer_updates_per_second = 60;
        let registers = Registers::default();

        let mut cpu = Self{
            registers,
            memory,
            i: 0,
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
            register_to_save_key: None,
            framebuffer,
            cycle_handler: CycleHandler{
                tick_duration: Duration::from_secs_f64(1.0 / ticks_per_second as f64),
                last_tick: Instant::now(),
                last_timer_update: Instant::now(),
                timer_update_duration: Duration::from_secs_f64(1.0 / timer_updates_per_second as f64),
            },
            pc: 0x200,
        };
        cpu.load_rom();
        cpu
    }
}

impl Cpu{
    pub fn load_rom(&mut self){
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("programs/space_invaders.ch8");
        let rom_data = fs::read(&path).expect("Failed to read ROM file");

        let start_address = 0x200;
        let end_address = start_address + rom_data.len();
        self.memory[start_address..end_address].copy_from_slice(&rom_data);
    }

    pub fn load_hex_sprites(&mut self){
        self.memory[HEX_SPRITE_START as usize..0x9F].copy_from_slice(&HEX_SPRITES);
    }

    pub fn update_timers(&mut self){
        let now = Instant::now();

        if now.duration_since(self.cycle_handler.last_timer_update) >= self.cycle_handler.timer_update_duration{
            if self.dt > 0{
                self.dt -= 1;
            }

            if self.st > 0{
                self.st -= 1;
            }
            self.cycle_handler.last_timer_update = now;
        }
    }

    pub fn waiting_key_pressed(&mut self, key: Keys){
        println!("KEY PRESS WAITED");
        let value: u8 = key.into();
        let register = self.registers.get_register_by_nibble(self.register_to_save_key.unwrap());
        self.register_to_save_key = None;
        *register = value;
    }

    pub fn tick<B: Backend>(&mut self, backend: &mut B){
        let now = Instant::now();

        if now.duration_since(self.cycle_handler.last_tick) >= self.cycle_handler.tick_duration{
            self.fetch(backend);
            self.cycle_handler.last_tick = Instant::now();
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
            0x0 => {
                if instruction.get_address() == 0x0E0{
                    self.framebuffer = [0; 64 * 32]
                } else {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp];
                }
            },
            0x1 => {
                let address = instruction.get_address();
                self.pc = address;
            },
            0x2 => {
                let address = instruction.get_address();
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = address;
            }
            0x3 => {
                let reg = self.registers.get_register_by_nibble(instruction.get_nibble(1));
                let byte = instruction.opcode[1];
                if *reg == byte{
                    self.pc += 2;
                }
            },
            0x4 => {
                let reg = self.registers.get_register_by_nibble(instruction.get_nibble(1));
                let byte = instruction.opcode[1];
                if *reg != byte{
                    self.pc += 2;
                }
            }
            0x5 => {
                let regx = *self.registers.get_register_by_nibble(instruction.get_nibble(1));
                let regy = *self.registers.get_register_by_nibble(instruction.get_nibble(2));
                if regx == regy{
                    self.pc += 2;
                }
            }
            0x6 => {
                let reg_nibble = instruction.get_nibble(1);
                let register = self.registers.get_register_by_nibble(reg_nibble);
                *register = instruction.opcode[1];
            },
            0x7 => {
                let reg_nibble = instruction.get_nibble(1);
                let register = self.registers.get_register_by_nibble(reg_nibble);
                let value = instruction.opcode[1];
                *register = register.wrapping_add(value);
            },
            0x8 => {
                let last_nibble = instruction.get_nibble(3);
                let regy = self.registers.get_register_value(instruction.get_nibble(2));
                let regx = self.registers.get_register_by_nibble(instruction.get_nibble(1));
                match last_nibble{
                    0 => *regx = regy,
                    1 => {
                        let or = *regx | regy;
                        *regx = or;
                    },
                    2 => {
                        let and = *regx & regy;
                        *regx = and;
                    },
                    3 => *regx ^= regy,
                    4 => {
                        let (val, overflow) = (*regx).overflowing_add(regy);
                        *regx = val;
                        self.registers.VF = overflow as u8;
                    },
                    5 => {
                        let carried = (*regx > regy) as u8;
                        *regx = (*regx).wrapping_sub(regy);
                        self.registers.VF = carried;
                    },
                    6 => {
                        let should = (*regx & 0x1) != 0;
                        *regx >>= 1;
                        self.registers.VF = should as u8;
                    },
                    7 => {
                        let should = (regy > *regx) as u8;
                        *regx = regy - *regx;
                        self.registers.VF = should;
                    },
                    8 => {
                        let should = (*regx & 0x80) != 0;
                        *regx <<= 1;
                        self.registers.VF = should as u8;
                    }
                    _ => ()
                }
            },
            0x9 => {
                let regy = self.registers.get_register_value(instruction.get_nibble(2));
                let regx = self.registers.get_register_by_nibble(instruction.get_nibble(1));
                if *regx != regy{
                    self.pc += 2;
                }
            },
            0xA => {
                let address = instruction.get_address();
                self.i = address;
            },
            0xB => {
                let address = instruction.get_address() + self.registers.V0 as u16;
                self.pc = address;
            },
            0xC => {
                let mut rng = rand::rng();
                let random_number: u8 = rng.random();
                let val = instruction.opcode[1];
                let vx = self.registers.get_register_by_nibble(instruction.get_nibble(1));
                *vx = val & random_number;
            }
            0xD => {
                self.registers.VF = 0;

                let x = *self.registers.get_register_by_nibble(instruction.get_nibble(1));
                let y = *self.registers.get_register_by_nibble(instruction.get_nibble(2));
                let sprite_height = instruction.get_nibble(3);
                for y_sprite_idx in (0..sprite_height).into_iter(){
                    let sprite_byte = self.memory[self.i as usize + y_sprite_idx as usize];
                    for x_sprite_idx in (0..8).into_iter(){
                        let x_pos = (x.wrapping_add(x_sprite_idx)) as usize % 64;
                        let y_pos = (y.wrapping_add(y_sprite_idx)) as usize % 32;
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
            },
            0xE => {
                let second_byte = instruction.opcode[1];
                let key_value = self.registers.get_register_value(instruction.get_nibble(1));
                let key = key_value.into();
                match second_byte{
                    0x9E => {
                        if backend.poll_key(key){
                            self.pc += 2;
                        }
                    },
                    0xA1 => {
                        if !backend.poll_key(key){
                            self.pc += 2;
                        }
                    }
                    _ => panic!()
                }
            },
            0xF => {
                let second_byte = instruction.opcode[1];
                let register = self.registers.get_register_by_nibble(instruction.get_nibble(1));
                match second_byte{
                    0x07 => {
                        *register = self.dt;
                    },
                    0x0A => {
                        self.register_to_save_key = Some(instruction.get_nibble(1));
                        backend.wait_for_key();
                    },
                    0x15 => {
                        self.dt = *register;
                    },
                    0x18 => {
                        self.st = *register;
                    },
                    0x1E => {
                        self.i += *register as u16;
                    },
                    0x29 => {
                        let val = *register;
                        let offset = val as u16 * HEX_SPRITE_LEN;
                        self.i = HEX_SPRITE_START + offset;
                    },
                    0x33 => {
                        let mut digits = Vec::new();
                        let mut value = *register;
                        if value == 0{
                            digits.push(value);
                        } else {
                            while value > 0{
                                digits.push(value % 10);
                                value /= 10;
                            }
                        }
                        digits.reverse();
                        for (idx, digit) in digits.into_iter().enumerate(){
                            self.memory[self.i as usize + idx] = digit;
                        }
                    },
                    0x55 => {
                        for (idx, nibble) in (0..=instruction.get_nibble(1)).enumerate(){
                            self.memory[self.i as usize + idx] = self.registers.get_register_value(nibble);
                        }
                    },
                    0x65 => {
                        let i = self.i as usize;
                        let final_register = instruction.get_nibble(1) as usize;

                        for idx in 0..=final_register{
                            let register = self.registers.get_register_by_nibble(idx as u8);
                            *register = self.memory[i + idx];
                        }
                    }
                    _ => panic!()
                }
            }
            _ => {
                panic!("Instruction {:?} not implemented", first_nibble)
            },
        }
    }
}
