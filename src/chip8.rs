#![allow(dead_code)]
#![allow(unused_variables)]
use std::default::Default;
use std::fs::read;
use std::ops::Add;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;
use std::ops::Shl;
use std::ops::Shr;
use std::ops::Sub;
use std::path::Path;

pub const MEMORY_SIZE: usize = 4 * 1024; // 0x1000 directions, from 0x0 to 0xFFF.
pub const DISPLAY_SIZE: usize = 64 * 32;
pub const REGISTER_SIZE: usize = 16;
pub const PROGRAM_MEMORY_START: usize = 0x200; // Programs usually start a 0x200.

fn high_nibble(b: u8) -> u8 {
    (b >> 4) & 0x0F
}

fn low_nibble(b: u8) -> u8 {
    b & 0x0F
}

fn low_and_high_nibbles(b: u8) -> [u8; 2] {
    [high_nibble(b), low_nibble(b)]
}

fn from_low_and_high(a: u8, b: u8) -> u8 {
    a << 4 | b
}

fn return_second<T>(_: T, x: T) -> T {
    x
}

#[derive(Debug)]
pub struct CHIP8 {
    memory: [u8; MEMORY_SIZE],
    display: [u8; DISPLAY_SIZE],
    register: [u8; REGISTER_SIZE],
    stack: Vec<u16>,
    pc: u16,
    index: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Default for CHIP8{
    fn default() -> CHIP8 {
    CHIP8 {
        memory: [0u8; MEMORY_SIZE],
        display: [0u8; DISPLAY_SIZE],
        register: [0u8; REGISTER_SIZE],
        stack: Vec::new(),
        pc: 0x0,
        index: 0x0,
        delay_timer: 0x0,
        sound_timer: 0x0,
    }
    }
}
impl CHIP8 {
   pub fn load_font(&mut self) {
        let font: [u8; 5 * 16] = [
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
        for (pos, e) in font.iter().enumerate() {
            self.memory[0x50+pos] = *e;
        }
    }

   pub fn load_testing_data(&mut self) {
        self.memory[0x200] = 0x00;
        self.memory[0x201] = 0xE0;
        self.pc = 0x200;
    }

    pub fn clear_screen(&mut self) {
        self.display = [0u8; DISPLAY_SIZE];
    }

    fn jump(&mut self, address: u8) {

    }

    fn set_register_to_immediate(&mut self, r: u8, n: u8) {
        self.register[r as usize] = n;
    }

    fn sum_register_with_immediate(&mut self, r: u8, n: u8) {
        self.register[r as usize] += n;
    }

    fn set_register_to_register(&mut self, r1: u8, r2: u8, op: fn(u8, u8) -> u8) {
        self.register[r1 as usize] = op(self.register[r1 as usize], self.register[r2 as usize]);
    }

    fn fetch(&mut self) -> [u8; 4] {
        let upc = self.pc as usize;
        let [fi, se] = low_and_high_nibbles(self.memory[upc]);
        let [th, fo] = low_and_high_nibbles(self.memory[upc+1]);
        self.pc += 2;
        [fi, se, th, fo]
    }

    pub fn execute(&mut self) {
        let instruction = self.fetch();
        // N = immediate
        // X, Y = register number (i.e. in 0XY0, X or Y could be 0-F)
        //
        match instruction {
            [0x0, 0x0, 0xE, 0x0] => self.clear_screen(),  // clear aka CLS
            [0x0, 0x0, 0xE, 0xE] => todo!(),              // return (exit subroutine) aka RTS
            [0x1, n1,  n2,  n3 ] => todo!(),              // jump NNN i.e. 12A0 = JUMP $2A8
            [0x2, n1,  n2,  n3 ] => todo!(),              // NNN (subroutine call)
            [0x3, x,   n1,  n2 ] => todo!(),              // if vx != NN then
            [0x4, x,   n1,  n2 ] => todo!(),              // if vx == NN then
            [0x5, x,   y,   0x0] => todo!(),              // if vx != vy then
            [0x6, x,   n1,  n2 ] => {
                self.set_register_to_immediate(x, from_low_and_high(n1, n2)) // vx := NN
            },
            [0x7, x,   n1,  n2 ] => {
                self.sum_register_with_immediate(x, from_low_and_high(n1, n2)) // vx += NN
            },
            [0x8, x,   y,   0x0] => {
                self.set_register_to_register(x, y, return_second) // vx := vy
            }
            [0x8, x,   y,   0x1] => {
                self.set_register_to_register(x, y, u8::bitor) // vx |= vy (bitwise OR)
            },
            [0x8, x,   y,   0x2] => {
                self.set_register_to_register(x, y, u8::bitand) // vx &= vy (bitwise AND)
            },
            [0x8, x,   y,   0x3] => {
                self.set_register_to_register(x, y, u8::bitxor) // vx ^= vy (bitwise XOR)
            },
            [0x8, x,   y,   0x4] => {
                // TODO: set carry
                self.set_register_to_register(x, y, u8::add) // vx += vy (vf = 1 on carry)
            },
            [0x8, x,   y,   0x5] => {
                // TODO: set borrow
                self.set_register_to_register(x, y, u8::sub) // vx -= vy (vf = 0 on borrow)
            }
            [0x8, x,   y,   0x6] => {
                // TODO: set least significant bit
                self.set_register_to_register(x, y, u8::shr) // vx >>= vy (vf = old least significant bit)
            },
            [0x8, x,   y,   0x7] => {
                // TODO: set borrow
                let f = |x: u8, y: u8| -> u8 {
                    u8::sub(y, x)
                };
                self.set_register_to_register(x, y, f) // vx =- vy (vf = 0 on borrow)
            },
            [0x8, x,   y,   0xE] => {
                self.set_register_to_register(x, y, u8::shl) // vx <<= vy (vf = old most significant bit)
            },
            [0x9, x,   y,   0x0] => todo!(),              // if vx == vy then
            [0xA, n1,  n2,  n3 ] => todo!(),              // i := NNN
            [0xB, n1,  n2,  n3 ] => todo!(),              // jump0 NNN (jump to address NNN + v0)
            [0xC, x,   n2,  n3 ] => todo!(),              // vx := random NN (random num 0-255 AND NN)
            [0xD, x,   y,   n  ] => todo!(),              // sprite vx vy N (vf = 1 on collision)
            [0xE, x,   0x9, 0xE] => todo!(),              // if vx -key then (is a key not pressed?)
            [0xE, x,   0xA, 0x1] => todo!(),              // if vx key then (is a key pressed?)
            [0xF, x,   0x0, 0x7] => todo!(),              // vx := delay
            [0xF, x,   0x0, 0xA] => todo!(),              // vx := key (wait for a keypress)
            [0xF, x,   0x1, 0x5] => todo!(),              // delay := vx
            [0xF, x,   0x1, 0x8] => todo!(),              // buzzer := vx
            [0xF, x,   0x1, 0xE] => todo!(),              // i += vx
            [0xF, x,   0x2, 0x9] => todo!(),              // i := hex vx (set i to a hex char)
            [0xF, x,   0x3, 0x3] => todo!(),              // bcd vx (decode vx into binary-coded decimal)
            [0xF, x,   0x5, 0x5] => todo!(),              // save vx (save v0-vx to i through (i+x))
            [0xF, x,   0x6, 0x5] => todo!(),              // load vx (load v0-vx to i through (i+x))
            _ => todo!(),
        }
    }
}
