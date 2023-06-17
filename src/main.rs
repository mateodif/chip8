#![feature(bigint_helper_methods)]

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub mod chip8;
use crate::chip8::CHIP8;

use chip8::Instruction;

fn main() {
    let mut chip = CHIP8::default();
    chip.load_font();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("CHIP-8",
                                        chip8::DISPLAY_HEIGHT as u32,
                                        chip8::DISPLAY_WIDTH as u32)
                                .position_centered()
                                .build()
                                .unwrap();

    let mut events = sdl_context.event_pump().unwrap();

    let mut waiting_for_keypress = None;

    loop {
        let instruction = match waiting_for_keypress {
            Some(instruction) => instruction,
            _ => chip.fetch()
        };
        for event in events.poll_iter() {
            match instruction {
                Instruction::DrawSprite { register1, register2, nibble } => {
                    let coord_x = chip.registers[register1 as usize] & (chip8::DISPLAY_HEIGHT - 1) as u8;
                    let coord_y = chip.registers[register2 as usize] & (chip8::DISPLAY_WIDTH - 1) as u8;
                    chip.registers[0xF as usize] = 0;

                    // reading the sprite
                    for row in 0..nibble {
                        // draw in screen starting from the coordinates
                        // each bit of the 'row' (byte) is a pixel

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

        // 60 FPS
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
