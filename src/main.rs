#![feature(bigint_helper_methods)]
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use std::path::Path;

pub mod chip8;
pub mod types;
use crate::chip8::CHIP8;

pub enum Kbd {
    Scode(Scancode),
    Quit,
}

fn get_scancode(event_pump: &mut sdl2::EventPump) -> Option<Kbd> {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => return Some(Kbd::Quit),
            Event::KeyDown {
                scancode: Some(scancode),
                ..
            } => {
                return Some(Kbd::Scode(scancode));
            }
            _ => continue,
        }
    }
    None
}

fn main() {
    let mut chip = CHIP8::default();
    chip.load_font();
    let path = Path::new("./resources/ibm_logo.ch8");
    chip.load_from_file(&path);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "CHIP-8",
            (chip8::DISPLAY_WIDTH * 10) as u32,
            (chip8::DISPLAY_HEIGHT * 10) as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(64, 32).unwrap();

    let mut events = sdl_context.event_pump().unwrap();

    'main: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        let instruction = chip.fetch();

        let scancode: Option<Kbd> = get_scancode(&mut events);
        match scancode {
            Some(Kbd::Quit) => break 'main,
            Some(Kbd::Scode(polled_scode)) => {
                chip.handle_keydown(Some(polled_scode));
            }
            None => {}
        }

        // println!("{:?}", instruction);

        chip.execute(instruction);

        canvas.set_draw_color(Color::GREEN);
        canvas
            .draw_points(chip.get_pixels_to_draw().as_slice())
            .unwrap();

        canvas.present();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 300));
    }
}
