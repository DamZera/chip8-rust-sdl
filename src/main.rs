extern crate sdl2;

use std::env;
use std::fs;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

const SCREEN_HEIGHT: u32 = 544; // 32*17
const SCREEN_WIDTH: u32 = 1088; // 64*17

const SPRITE_0: u16 = 0x1001;
const SPRITE_1: u16 = 0x1002;
const SPRITE_2: u16 = 0x1003;
const SPRITE_3: u16 = 0x1004;
const SPRITE_A: u16 = 0x1005;
const SPRITE_B: u16 = 0x1006;
const SPRITE_C: u16 = 0x1007;
const SPRITE_D: u16 = 0x1008;

const sprite_0: [u8; 5] = [
    0b01100000,
    0b10010000,
    0b10010000,
    0b10010000,
    0b01100000
];

const sprite_1: [u8; 5] = [
    0b01100000,
    0b00100000,
    0b00100000,
    0b00100000,
    0b01110000
];

const sprite_2: [u8; 5] = [
    0b11100000,
    0b00010000,
    0b00100000,
    0b01000000,
    0b1111000
];

const sprite_3: [u8; 5] = [
    0b11000011,
    0b01100110,
    0b00111100,
    0b01100110,
    0b10000001
];

const sprite_a: [u8; 5] = [
    0b11000011,
    0b01100110,
    0b00111100,
    0b01100110,
    0b10000001
];

const sprite_b: [u8; 5] = [
    0b11000011,
    0b01100110,
    0b00111100,
    0b01100110,
    0b10000001
];

const sprite_c: [u8; 5] = [
    0b11000011,
    0b01100110,
    0b00111100,
    0b01100110,
    0b10000001
];

const sprite_d: [u8; 5] = [
    0b11000011,
    0b01100110,
    0b00111100,
    0b01100110,
    0b10000001
];

#[derive(Debug)]
struct Chip8 {
    // define registers
    v: [u8; 16],
    mem: [u8; 0x1000],
    stack: [usize; 16],

    pc: usize,
    sp: usize, // stack position
    reg_i: u16,
    timer_delay: u8,
}

fn build_default_chip8() -> Chip8 {
    Chip8 {
        // define registers
        v: [0, 0, 0, 0, 0, 0, 0, 0, 0 ,0 ,0 ,0, 0, 0, 0, 0],
        mem: [0; 0x1000],
        stack: [0, 0, 0, 0, 0, 0, 0, 0, 0 ,0 ,0 ,0, 0, 0, 0, 0],

        pc: 0x200,
        sp: 0, // stack position
        reg_i: 0,
        timer_delay: 0,
    }
}

impl Chip8 {
    fn execute_command(&mut self) -> bool {
        let opcode : u16 = (self.mem[self.pc] as u16)<<(4*2) | self.mem[self.pc+1] as u16;
        print!("[{:04x}] {:04x} ", self.pc, opcode); // issue in addr
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
                println!("ADD V{} += {:#02x}", vx, nn);
                match self.v[vx].checked_add((nn) as u8) {
                    Some(val) => {
                        self.v[vx] = val;
                    }
                    None => {
                        self.v[vx] = 0;
                        println!("overflow ignore!");
                    }
                };
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
                        println!("V{} += V{}", vx, vy);
                        self.v[vx] += self.v[vy];
                    }
                    0x5 => {
                        println!("V{} -= V{}", vx, vy);
                        self.v[vx] -= self.v[vy];
                    }
                    0x6 => {
                        println!("V{} >>= 1", vx);
                        self.v[vx] >>=1
                    }
                    0x7 => {
                        println!("V{} = V{} - V{}", vx, vy, vx);
                        self.v[vx] = self.v[vy] - self.v[vx];
                    }
                    0xe => {
                        println!("V{} <<= 1", vx);
                        self.v[vx] <<=1;
                    }
                    _ => println!("ERROR 8"),
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
            0xc => println!("RAND todo"),
            0xd => {
                println!("DRAW (V{}, V{}, {})", vx, vy, n);

            }
            0xe => {
                if (nn) == 0x9e {
                    println!("SKIP if (KEY == V{})", vy);
                    //panic!("opcode 0xe9e not implemented");
                }
                else if (nn) == 0xa1 {
                    println!("SKIP if (KEY != V{})", vy);
                    //panic!("opcode 0xea1 not implemented");
                }
                else {
                    println!("ERROR E");
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
                        // TODO
                    }
                    0x15 => {
                        println!("delay_timer(V{} = {})", vx, self.v[vx]);
                        self.timer_delay = self.v[vx];
                    }
                    0x18 => println!("SOUND !!"),
                    0x1e => {
                        println!("I += V{}", vx);
                        self.reg_i += self.v[vx] as u16;
                    }
                    0x29 => {
                        println!("I = sprite_addr[V{}]", vx);
                        // TODO implement this part
                        println!("TODO : sprite {}", self.v[vx]);
                        self.reg_i = SPRITE_D;
                    }
                    0x33 => println!("set_BCD(V{})*(I+0) = BCD(3);*(I+1) = BCD(2);*(I+2) = BCD(1);", vx),
                    0x55 => println!("reg_dump(V{}, &I)", vx),
                    0x65 => println!("reg_dump(V{}, &I)", vx),
                    _ => println!("ERROR F")
                }
            }
            _ => println!("ERROR"),
        }
        self.pc += 2;
        if self.pc + 1 >= self.mem.len(){
            return true;
        }

        false
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];
    println!("Rom path is {rom_path}");

    let rom_bytes : Vec<u8> = fs::read(rom_path).expect("Bytes");
    let mut end = false;

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

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        end = chip.execute_command();
        if instant.elapsed() >= tick_duration {
            if chip.timer_delay > 0 {
                chip.timer_delay -= 1;
            }
            instant = Instant::now();
        }

        canvas.present();
        ::std::thread::sleep(tick_duration);
    }
}