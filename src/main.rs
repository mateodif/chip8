#![feature(bigint_helper_methods)]

use std::path::Path;
use sdl2::pixels::Color;
use sdl2::event::Event;

pub mod chip8;
use crate::chip8::CHIP8;

use chip8::Instruction;

fn main() {
    let mut chip = CHIP8::default();
    chip.load_font();
    let path = Path::new("./resources/test_opcode.ch8");
    chip.load_from_file(&path);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("CHIP-8",
                                        (chip8::DISPLAY_HEIGHT * 8) as u32,
                                        (chip8::DISPLAY_WIDTH * 8) as u32)
                                .position_centered()
                                .build()
                                .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(64, 32).unwrap();

    let mut events = sdl_context.event_pump().unwrap();

    let mut waiting_for_keypress = None;

    loop {
        canvas.set_draw_color(Color::RGB(97, 134, 169));
        canvas.clear();

        let instruction = match waiting_for_keypress {
            Some(instruction) => instruction,
            _ => chip.fetch()
        };

        println!("{:?}", instruction);

        for event in events.poll_iter() {
            match instruction {
                Instruction::DrawSprite { register1, register2, nibble } => {
                    let coord_x = (chip.registers[register1 as usize] & (chip8::DISPLAY_HEIGHT - 1) as u8) as usize;
                    let coord_y = (chip.registers[register2 as usize] & (chip8::DISPLAY_WIDTH - 1) as u8) as usize;
                    chip.registers[0xF as usize] = 0;

                    for byte in 0..(nibble as usize) {
                        let y = (coord_y + byte) % chip8::DISPLAY_HEIGHT;
                        for bit in 0..8 {
                            let x = (coord_x + bit) % chip8::DISPLAY_WIDTH;
                            let sprite_pixel = (chip.memory[chip.index as usize + byte] >> (7 - bit)) & 1;
                            let display_pixel = chip.display[x][y];

                            chip.display[x][y] ^= sprite_pixel;

                            if display_pixel == 1 && sprite_pixel == 1 {
                                chip.registers[0xF as usize] = 1;
                            }
                        }
                    }

                },
                Instruction::SkipIfKeyPressed { register } => {
                    match event {
                        Event::KeyDown { keycode: Some(keycode), .. } => {
                            if chip.registers[register as usize] == keycode as u8 {
                                chip.skip();
                            }
                        },
                        _ => continue,
                    };
                },
                Instruction::SkipIfKeyNotPressed { register } => {
                    match event {
                        Event::KeyDown { keycode: Some(keycode), .. } => {
                            if chip.registers[register as usize] != keycode as u8 {
                                chip.skip();
                            }
                        },
                        _ => continue,
                    };
                },
                Instruction::WaitForKeyPress { register } => {
                    waiting_for_keypress = Some(instruction);
                    match event {
                        Event::KeyDown { keycode: Some(keycode), .. } => {
                            chip.registers[register as usize] = keycode as u8;
                            waiting_for_keypress = None;
                            continue
                        },
                        _ => continue,
                    };
                },
                _ => {
                    chip.execute(instruction);
                    continue
                },
            }
        }

        canvas.set_draw_color(Color::RGB(33, 41, 70));
        for (x, row) in chip.display.iter().enumerate() {
            for (y, _) in row.iter().enumerate() {
                if chip.display[x][y] == 1 {
                    canvas.draw_point((x as i32, y as i32)).unwrap();
                }
            }
        }
        canvas.present();

        // 60 FPS
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
