#![feature(bigint_helper_methods)]

use std::path::Path;
use sdl2::pixels::Color;
use sdl2::event::Event;

pub mod chip8;
use crate::chip8::CHIP8;

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

    loop {
        canvas.set_draw_color(Color::RGB(97, 134, 169));
        canvas.clear();


        let mut keydown_event = None;

        for event in events.poll_iter() {
            match event {
                Event::KeyDown { keycode: Some(keycode), .. } => keydown_event = Some(keycode),
                _ => {}
            }
        }

        let instruction = chip.fetch();
        println!("{:?}", instruction);
        chip.handle_keydown(keydown_event);
        chip.execute(instruction);

        canvas.set_draw_color(Color::RGB(33, 41, 70));

        for (x, row) in chip.get_display().iter().enumerate() {
            for (y, _) in row.iter().enumerate() {
                if chip.get_display()[x][y] == 1 {
                    canvas.draw_point((x as i32, y as i32)).unwrap();
                }
            }
        }

        canvas.present();

        // 60 FPS
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 30));
    }
}
