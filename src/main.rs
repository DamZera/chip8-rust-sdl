extern crate sdl2;

use std::env;
use std::fs;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = &args[1];
    println!("Rom path is {rom_path}");

    // define registers
    let mut v: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0 ,0 ,0 ,0, 0, 0, 0, 0];
    let mut mem: [u8; 0x1000] = [0; 0x1000];
    let mut stack: [usize; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0 ,0 ,0 ,0, 0, 0, 0, 0];

    let mut end = false;
    let mut pc = 0x200;
    let mut sp = 0; // stack position

    let rom_bytes : Vec<u8> = fs::read(rom_path).expect("Bytes");

    // store rom in mem
    let mut i = 0x200;
    for byte in rom_bytes {
        mem[i] = byte;
        i += 1;
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
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

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    // CHIP8
    while !end {
        let opcode : u16 = (mem[pc] as u16)<<(4*2) | mem[pc+1] as u16;
        print!("[{:04x}] {:04x} ", pc, opcode); // issue in addr
        let nnn = opcode & 0xfff;
        let nn  = opcode & 0xff;
        let n   = opcode & 0xf;
        let vx  = (opcode>>(4*2)) & 0xf;
        let vy  = (opcode>>4) & 0xf;

        match opcode>>(4*3) {
            0x0 => {
                if opcode == 0x00ee {
                    println!("RET");
                    sp -= 1;
                    pc = stack[sp];
                    continue;
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
                pc = nnn as usize;
                continue;
            }
            0x2 => {
                println!("CALL {:04x}", nnn);
                stack[sp] = pc + 2;
                sp += 1;
                pc = nnn as usize;
                continue;
            }
            0x3 => {
                println!("SKIP if V{} == {:#02x}", vx, nn);
                if u16::from(v[(vx) as usize]) == nn {
                    pc += 2;
                }
            }
            0x4 => {
                println!("SKIP if V{} != {:#02x}", vx, nn);
                if u16::from(v[(vx) as usize]) != nn {
                    pc += 2;
                }
            }
            0x5 => {
                println!("SKIP if V{} == V{}", vx, vy);
                if v[(vx) as usize] == v[(vy) as usize] {
                    pc += 2;
                }
            }
            0x6 => {
                println!("SET V{} = {:#02x}", vx, nn);
                v[(vx) as usize] = (nn) as u8;
            }
            0x7 => {
                println!("ADD V{} += {:#02x}", vx, nn);
                match v[(vx) as usize].checked_add((nn) as u8) {
                    Some(val) => {
                        v[(vx) as usize] = val;
                    }
                    None => {
                        v[(vx) as usize] = 0;
                        println!("overflow ignore!");
                    }
                };
            }
            0x8 => {
                match opcode&0xf {
                    0x0 => {
                        println!("SET V{} = V{}", vx, vy);
                        v[(vx) as usize] = v[(vy) as usize];
                    }
                    0x1 => {
                        println!("V{} |= V{}", vx, vy);
                        v[(vx) as usize] |= v[(vy) as usize];
                    }
                    0x2 => {
                        println!("V{} &= V{}", vx, vy);
                        v[(vx) as usize] &= v[(vy) as usize];
                    }
                    0x3 => {
                        println!("V{} ^= V{}", vx, vy);
                        v[(vx) as usize] ^= v[(vy) as usize];
                    }
                    0x4 => {
                        println!("V{} += V{}", vx, vy);
                        v[(vx) as usize] += v[(vy) as usize];
                    }
                    0x5 => {
                        println!("V{} -= V{}", vx, vy);
                        v[(vx) as usize] -= v[(vy) as usize];
                    }
                    0x6 => {
                        println!("V{} >>= 1", vx);
                        v[(vx) as usize] >>=1
                    }
                    0x7 => {
                        println!("V{} = V{} - V{}", vx, vy, vx);
                        v[(vx) as usize] = v[(vy) as usize] - v[(vx) as usize];
                    }
                    0xe => {
                        println!("V{} <<= 1", vx);
                        v[(vx) as usize] <<=1;
                    }
                    _ => println!("ERROR 8"),
                }
            }
            0x9 => {
                println!("SKIP V{} != V{}", vx, vy);
                if v[(vx) as usize] != v[(vy) as usize] {
                    pc += 2;
                }
            }
            0xa => {
                println!("SET I = {}", nnn);
                panic!("opcode 0xa not implemented");
            }
            0xb => {
                println!("PC = V0 + {}", nnn);
                pc = (v[0] as u16 + nnn) as usize;
            }
            0xc => println!("RAND todo"),
            0xd => println!("DRAW (V{}, V{}, {})", vx, vy, n),
            0xe => {
                if (opcode&0xff) == 0x9e {
                    println!("SKIP if (KEY == V{})", vy);
                    //panic!("opcode 0xe9e not implemented");
                }
                else if (opcode&0xff) == 0xa1 {
                    println!("SKIP if (KEY != V{})", vy);
                    //panic!("opcode 0xea1 not implemented");
                }
                else {
                    println!("ERROR E");
                }
            }
            0xf => {
                match opcode&0xff {
                    0x07 => println!("V{} = get_delay()", vx),
                    0x0a => println!("V{} = get_key()", vx),
                    0x15 => println!("delay_timer(V{})", vx),
                    0x18 => println!("SOUND !!"),
                    0x1e => println!("I += V{}", vx),
                    0x29 => println!("I = sprite_addr[V{}]", vx),
                    0x33 => println!("set_BCD(V{})*(I+0) = BCD(3);*(I+1) = BCD(2);*(I+2) = BCD(1);", vx),
                    0x55 => println!("reg_dump(V{}, &I)", vx),
                    0x65 => println!("reg_dump(V{}, &I)", vx),
                    _ => println!("ERROR F")
                }
            }
            _ => println!("ERROR"),
        }
        pc += 2;
        if pc + 1 >= mem.len(){
            end = true;
        }
    }
}
