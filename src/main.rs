extern crate sdl2;

use std::env;
use std::fs;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, KeyboardState, Scancode};
use sdl2::rect::Rect;

use std::time::{Duration, Instant};

mod core;

const CHIP8_HEIGHT: usize = 32;
const CHIP8_WIDTH: usize = 64;

const SCALE_FACTOR: u32 = 17;

const SCREEN_HEIGHT: u32 = CHIP8_HEIGHT as u32 * SCALE_FACTOR; // 32*17
const SCREEN_WIDTH: u32 = CHIP8_WIDTH as u32 * SCALE_FACTOR; // 64*17

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

    let mut chip : core::Chip8 = core::build_default_chip8();

    // store rom in mem
    let mut i = 0x200;
    for byte in rom_bytes {
        chip.mem[i] = byte;
        i += 1;
    }

    let mut instant = Instant::now();
    let tick_duration = Duration::from_millis(1000/60);

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