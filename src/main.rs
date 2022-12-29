extern crate sdl2;

use std::env;
use std::fs;
use rand::Rng;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, KeyboardState, Scancode};
use sdl2::rect::Rect;

use std::time::{Duration, Instant};

const CHIP8_HEIGHT: usize = 32;
const CHIP8_WIDTH: usize = 64;

const SCALE_FACTOR: u32 = 17;

const SCREEN_HEIGHT: u32 = CHIP8_HEIGHT as u32 * SCALE_FACTOR; // 32*17
const SCREEN_WIDTH: u32 = CHIP8_WIDTH as u32 * SCALE_FACTOR; // 64*17

pub const FONT_SET: [u8; 80] = [
    0xF0,
    0x90,
    0x90,
    0x90,
    0xF0,
    0x20,
    0x60,
    0x20,
    0x20,
    0x70,
    0xF0,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x10,
    0xF0,
    0x10,
    0xF0,
    0x90,
    0x90,
    0xF0,
    0x10,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x10,
    0x20,
    0x40,
    0x40,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0x80,
    0x80,
    0xF0,
    0xE0,
    0x90,
    0x90,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0x80,
];

#[derive(Debug)]
struct Chip8 {
     // define registers, stack, vram and ram
    v: [u8; 16],
    mem: [u8; 0x1000],
    vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    stack: [usize; 16],

    keypad: [u8; 16],

    pc: usize, // pointer counter
    sp: usize, // stack position
    reg_i: u16,
    timer_delay: u8,
    vram_changed: bool,
}

fn build_default_chip8() -> Chip8 {
    let mut chip = Chip8 {
        v: [0, 0, 0, 0, 0, 0, 0, 0, 0 ,0 ,0 ,0, 0, 0, 0, 0],
        mem: [0; 0x1000],
        vram: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
        stack: [0, 0, 0, 0, 0, 0, 0, 0, 0 ,0 ,0 ,0, 0, 0, 0, 0],

        keypad: [0; 16],

        pc: 0x200,
        sp: 0,
        reg_i: 0,
        timer_delay: 0,
        vram_changed: false,
    };

    for i in 0..FONT_SET.len() {
        chip.mem[i] = FONT_SET[i];
    }

    chip
}

impl Chip8 {
    fn execute_command(&mut self) -> bool {
        let opcode : u16 = (self.mem[self.pc] as u16)<<(4*2) | self.mem[self.pc+1] as u16;
        print!("[{:04x}] {:04x} ", self.pc, opcode);
        let nnn = opcode & 0xfff;
        let nn  = opcode & 0xff;
        let n   = opcode & 0xf;
        let vx: usize = ((opcode>>(4*2)) & 0xf) as usize;
        let vy: usize  = ((opcode>>4) & 0xf) as usize;

        match opcode>>(4*3) {
            0x0 => {
                if opcode == 0x00ee {
                    println!("RET");
                    self.sp -= 1;
                    self.pc = self.stack[self.sp];
                    return false;
                }
                else if opcode == 0x00e0 {
                    println!("CLR_DSP");
                    for y in 0..CHIP8_HEIGHT {
                        for x in 0..CHIP8_WIDTH {
                            self.vram[y][x] = 0;
                        }
                    }
                    self.vram_changed = true;
                }
                else {
                    println!("ERROR 0")
                }
            }
            0x1 => {
                println!("JMP {:04x}", nnn);
                self.pc = nnn as usize;
                return false;
            }
            0x2 => {
                println!("CALL {:04x}", nnn);
                self.stack[self.sp] = self.pc + 2;
                self.sp += 1;
                self.pc = nnn as usize;
                return false;
            }
            0x3 => {
                println!("SKIP if V{} == {:#02x}", vx, nn);
                if u16::from(self.v[vx]) == nn {
                    self.pc += 2;
                }
            }
            0x4 => {
                println!("SKIP if V{} != {:#02x}", vx, nn);
                if u16::from(self.v[vx]) != nn {
                    self.pc += 2;
                }
            }
            0x5 => {
                println!("SKIP if V{} == V{}", vx, vy);
                if self.v[vx] == self.v[vy] {
                    self.pc += 2;
                }
            }
            0x6 => {
                println!("SET V{} = {:#02x}", vx, nn);
                self.v[vx] = (nn) as u8;
            }
            0x7 => {
                println!("ADD V{}({}) += {:#02x}", vx, self.v[vx], nn);
                self.v[vx] = self.v[vx].checked_add(nn as u8).unwrap_or(((self.v[vx] as u16 + nn ) % 255) as u8);
            }
            0x8 => {
                match n {
                    0x0 => {
                        println!("SET V{} = V{}", vx, vy);
                        self.v[vx] = self.v[vy];
                    }
                    0x1 => {
                        println!("V{} |= V{}", vx, vy);
                        self.v[vx] |= self.v[vy];
                    }
                    0x2 => {
                        println!("V{} &= V{}", vx, vy);
                        self.v[vx] &= self.v[vy];
                    }
                    0x3 => {
                        println!("V{} ^= V{}", vx, vy);
                        self.v[vx] ^= self.v[vy];
                    }
                    0x4 => {
                        println!("V{}({}) += V{}({})", vx, self.v[vx], vy, self.v[vy]);
                        let reg = (self.v[vx] as u16) + (self.v[vy] as u16);
                        self.v[vx] = reg as u8;
                        self.v[0xf] = if reg > 0xff { 1 } else { 0 };
                    }
                    0x5 => {
                        println!("V{}({}) -= V{}({})", vx, self.v[vx], vy, self.v[vy]);
                        self.v[vx] = self.v[vx].checked_sub(self.v[vy]).unwrap_or(255 - self.v[vy]);
                        self.v[0xf] = if self.v[vx] > self.v[vy] { 1 } else { 0 };
                    }
                    0x6 => {
                        println!("V{} >>= 1", vx);
                        self.v[vx] >>=1;
                        self.v[0xf] = self.v[vx] & 1;
                    }
                    0x7 => {
                        println!("V{} = V{} - V{}", vx, vy, vx);
                        self.v[vx] = self.v[vy] - self.v[vx];
                        self.v[0xf] = if self.v[vy] > self.v[vx] { 1 } else { 0 };
                    }
                    0xe => {
                        println!("V{} <<= 1", vx);
                        self.v[0xf] = (self.v[vx] & 0b10000000) >> 7;
                        self.v[vx] <<=1;
                    }
                    _ => panic!("ERROR unknown OPCODE {}", opcode),
                }
            }
            0x9 => {
                println!("SKIP V{} != V{}", vx, vy);
                if self.v[vx] != self.v[vy] {
                    self.pc += 2;
                }
            }
            0xa => {
                println!("SET I = {}", nnn);
                self.reg_i = nnn;
            }
            0xb => {
                println!("PC = V0 + {}", nnn);
                self.pc = (self.v[0] as u16 + nnn) as usize;
            }
            0xc => {
                println!("V{} = RAND & NN({})", vx, nn);
                let mut rng = rand::thread_rng();
                self.v[vx] = (rng.gen_range(0..256) as u8) & nn as u8;
            }
            0xd => {
                println!("DRAW (V{}, V{}, {})", vx, vy, n);
                self.v[0xf] = 0;
                for byte in 0..n {
                    let y = (self.v[vy] as usize + byte as usize) % CHIP8_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.v[vx] as usize + bit) % CHIP8_WIDTH;
                        let color = (self.mem[self.reg_i as usize + byte as usize] >> (7 - bit)) & 1;
                        self.v[0xf] |= color & self.vram[y][x];
                        self.vram[y][x] ^= color;

                    }
                }
                self.vram_changed = true;
            }
            0xe => {
                if (nn) == 0x9e {
                    println!("SKIP if (KEY == V{})", vx);
                    if self.keypad[self.v[vx] as usize] != 0 {
                        self.pc += 2;
                    }
                }
                else if (nn) == 0xa1 {
                    println!("SKIP if (KEY != V{})", vx);
                    if self.keypad[self.v[vx] as usize] != 1 {
                        self.pc += 2;
                    }
                }
                else {
                    panic!("ERROR unknown OPCODE {}", opcode);
                }
            }
            0xf => {
                match nn {
                    0x07 => {
                        println!("V{} = get_delay() {}", vx, self.timer_delay);
                        self.v[vx] = self.timer_delay;
                    }
                    0x0a => {
                        println!("V{} = get_key()", vx);
                        for i in 0..0xF {
                            if self.keypad[i] != 0 {
                                self.v[vx] = i as u8;
                                break;
                            }
                        }
                    }
                    0x15 => {
                        println!("delay_timer(V{} = {})", vx, self.v[vx]);
                        self.timer_delay = self.v[vx];
                    }
                    0x18 => println!("SOUND !!"),
                    0x1e => {
                        println!("I += V{}", vx);
                        self.reg_i += self.v[vx] as u16;
                        self.v[0xf] = if self.reg_i > 0xf00 { 1 } else { 0 };
                    }
                    0x29 => {
                        println!("I = sprite_addr[V{}]", vx);
                        self.reg_i = (self.v[vx] as u16) * 5;
                    }
                    0x33 => {
                        println!("set_BCD(V{}:{})*(I+0) = BCD(3);*(I+1) = BCD(2);*(I+2) = BCD(1);", vx, self.v[vx]);
                        let i = self.reg_i as usize;
                        self.mem[i] = self.v[vx] / 100;
                        self.mem[i + 1] = self.v[vx] % 100 / 10;
                        self.mem[i + 2] = self.v[vx] % 10;
                    }
                    0x55 => {
                        println!("reg_dump(V{}, &I)", vx);
                        for byte in 0..(vx+1) {
                            self.mem[self.reg_i as usize + byte as usize] = self.v[byte];
                        }
                    }
                    0x65 => {
                        println!("reg_load(V{}, &I)", vx);
                        for byte in 0..(vx+1) {
                            self.v[byte] = self.mem[self.reg_i as usize + byte as usize];
                        }
                    }
                    _ => panic!("ERROR unknown OPCODE {}", opcode)
                }
            }
            _ => panic!("ERROR unknown OPCODE {}", opcode),
        }
        self.pc += 2;
        if self.pc + 1 >= self.mem.len(){
            return true;
        }

        false
    }
}

fn color(value: u8) -> Color {
    if value == 0 {
        Color::RGB(0, 0, 0)
    } else {
        Color::RGB(255, 255, 255)
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];
    println!("Rom path is {rom_path}");

    let rom_bytes : Vec<u8> = fs::read(rom_path).expect("Bytes");

    let mut chip : Chip8 = build_default_chip8();

    // store rom in mem
    let mut i = 0x200;
    for byte in rom_bytes {
        chip.mem[i] = byte;
        i += 1;
    }

    let mut instant = Instant::now();
    let tick_duration = Duration::from_millis(1000/60);

    //println!("Chip8 data {:?}", chip);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                _ => {},
            };
        }

        // Keypad is hex values 0-9 A-F
        let key_state = KeyboardState::new(&mut event_pump);
        chip.keypad[0x0] = key_state.is_scancode_pressed(Scancode::Num0) as u8;
        chip.keypad[0x1] = key_state.is_scancode_pressed(Scancode::Num1) as u8;
        chip.keypad[0x2] = key_state.is_scancode_pressed(Scancode::Num2) as u8;
        chip.keypad[0x3] = key_state.is_scancode_pressed(Scancode::Num3) as u8;
        chip.keypad[0x4] = key_state.is_scancode_pressed(Scancode::Num4) as u8;
        chip.keypad[0x5] = key_state.is_scancode_pressed(Scancode::Num5) as u8;
        chip.keypad[0x6] = key_state.is_scancode_pressed(Scancode::Num6) as u8;
        chip.keypad[0x7] = key_state.is_scancode_pressed(Scancode::Num7) as u8;
        chip.keypad[0x8] = key_state.is_scancode_pressed(Scancode::Num8) as u8;
        chip.keypad[0x9] = key_state.is_scancode_pressed(Scancode::Num9) as u8;
        chip.keypad[0xA] = key_state.is_scancode_pressed(Scancode::A) as u8;
        chip.keypad[0xB] = key_state.is_scancode_pressed(Scancode::B) as u8;
        chip.keypad[0xC] = key_state.is_scancode_pressed(Scancode::C) as u8;
        chip.keypad[0xD] = key_state.is_scancode_pressed(Scancode::D) as u8;
        chip.keypad[0xE] = key_state.is_scancode_pressed(Scancode::E) as u8;
        chip.keypad[0xF] = key_state.is_scancode_pressed(Scancode::F) as u8;

        chip.execute_command();

        if chip.vram_changed {
            for (y, row) in chip.vram.iter().enumerate() {
                for (x, &col) in row.iter().enumerate() {
                    let x = (x as u32) * SCALE_FACTOR;
                    let y = (y as u32) * SCALE_FACTOR;
    
                    canvas.set_draw_color(color(col));
                    let _ = canvas
                        .fill_rect(Rect::new(x as i32, y as i32, SCALE_FACTOR, SCALE_FACTOR));
                }
            }
            canvas.present();
            chip.vram_changed = false;
        }

        if instant.elapsed() >= tick_duration {
            if chip.timer_delay > 0 {
                chip.timer_delay -= 1;
            }
            instant = Instant::now();
        }

        ::std::thread::sleep_ms(2);
    }
}