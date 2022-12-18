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

impl Default for CHIP8 {
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
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        for (pos, e) in font.iter().enumerate() {
            self.memory[0x50 + pos] = *e;
        }
    }

    pub fn load_from_slice(&mut self, slice: &[u8]) {
        for (i, _) in slice.iter().enumerate() {
            self.memory[i + PROGRAM_MEMORY_START] = slice[i];
        }
    }
    pub fn load_from_file(&mut self, path: &Path) {
        let path = path.canonicalize().unwrap();
        let file = read(path).unwrap();
        for (i, _) in file.iter().enumerate() {
            self.memory[i + PROGRAM_MEMORY_START] = file[i];
        }
    }

    pub fn clear_screen(&mut self) {
        self.display = [0u8; DISPLAY_SIZE];
    }

    fn jump(&mut self, address: u8) {}

    fn set_register_to_immediate(&mut self, r: u8, n: u8) {
        self.register[r as usize] = n;
    }

    fn sum_register_with_immediate(&mut self, r: u8, n: u8) {
        self.register[r as usize] += n;
    }

    fn set_register_from_register(&mut self, r1: u8, r2: u8, op: fn(u8, u8) -> u8) {
        self.register[r1 as usize] = op(self.register[r1 as usize], self.register[r2 as usize]);
    }

    fn fetch(&mut self) -> [u8; 4] {
        let upc = self.pc as usize;
        let [first_nibble, second_nibble] = low_and_high_nibbles(self.memory[upc]);
        let [third_nibble, fourth_nibble] = low_and_high_nibbles(self.memory[upc + 1]);
        self.pc += 2;
        [first_nibble, second_nibble, third_nibble, fourth_nibble]
    }

    pub fn execute(&mut self) {
        let instruction = self.fetch();
        // N = immediate
        // X, Y = register number (i.e. in 0XY0, X or Y could be 0-F)
        //
        match instruction {
            [0x0, 0x0, 0xE, 0x0] => self.clear_screen(), // clear aka CLS
            [0x0, 0x0, 0xE, 0xE] => todo!(),             // return (exit subroutine) aka RTS
            [0x1, n1, n2, n3] => todo!(),                // jump NNN i.e. 12A0 = JUMP $2A8
            [0x2, n1, n2, n3] => todo!(),                // NNN (subroutine call)
            [0x3, x, n1, n2] => todo!(),                 // if vx != NN then
            [0x4, x, n1, n2] => todo!(),                 // if vx == NN then
            [0x5, x, y, 0x0] => todo!(),                 // if vx != vy then
            [0x6, x, n1, n2] => {
                self.set_register_to_immediate(x, from_low_and_high(n1, n2)) // vx := NN
            }
            [0x7, x, n1, n2] => {
                self.sum_register_with_immediate(x, from_low_and_high(n1, n2)) // vx += NN
            }
            [0x8, x, y, 0x0] => {
                self.set_register_from_register(x, y, return_second) // vx := vy
            }
            [0x8, x, y, 0x1] => {
                self.set_register_from_register(x, y, u8::bitor) // vx |= vy (bitwise OR)
            }
            [0x8, x, y, 0x2] => {
                self.set_register_from_register(x, y, u8::bitand) // vx &= vy (bitwise AND)
            }
            [0x8, x, y, 0x3] => {
                self.set_register_from_register(x, y, u8::bitxor) // vx ^= vy (bitwise XOR)
            }
            [0x8, x, y, 0x4] => {
                // TODO: set carry
                self.set_register_from_register(x, y, u8::add) // vx += vy (vf = 1 on carry)
            }
            [0x8, x, y, 0x5] => {
                // TODO: set borrow
                self.set_register_from_register(x, y, u8::sub) // vx -= vy (vf = 0 on borrow)
            }
            [0x8, x, y, 0x6] => {
                // TODO: set least significant bit
                self.set_register_from_register(x, y, u8::shr) // vx >>= vy (vf = old least significant bit)
            }
            [0x8, x, y, 0x7] => {
                // TODO: set borrow
                let f = |x: u8, y: u8| -> u8 { u8::sub(y, x) };
                self.set_register_from_register(x, y, f) // vx =- vy (vf = 0 on borrow)
            }
            [0x8, x, y, 0xE] => {
                self.set_register_from_register(x, y, u8::shl) // vx <<= vy (vf = old most significant bit)
            }
            [0x9, x, y, 0x0] => todo!(),   // if vx == vy then
            [0xA, n1, n2, n3] => todo!(),  // i := NNN
            [0xB, n1, n2, n3] => todo!(),  // jump0 NNN (jump to address NNN + v0)
            [0xC, x, n2, n3] => todo!(),   // vx := random NN (random num 0-255 AND NN)
            [0xD, x, y, n] => todo!(),     // sprite vx vy N (vf = 1 on collision)
            [0xE, x, 0x9, 0xE] => todo!(), // if vx -key then (is a key not pressed?)
            [0xE, x, 0xA, 0x1] => todo!(), // if vx key then (is a key pressed?)
            [0xF, x, 0x0, 0x7] => todo!(), // vx := delay
            [0xF, x, 0x0, 0xA] => todo!(), // vx := key (wait for a keypress)
            [0xF, x, 0x1, 0x5] => todo!(), // delay := vx
            [0xF, x, 0x1, 0x8] => todo!(), // buzzer := vx
            [0xF, x, 0x1, 0xE] => todo!(), // i += vx
            [0xF, x, 0x2, 0x9] => todo!(), // i := hex vx (set i to a hex char)
            [0xF, x, 0x3, 0x3] => todo!(), // bcd vx (decode vx into binary-coded decimal)
            [0xF, x, 0x5, 0x5] => todo!(), // save vx (save v0-vx to i through (i+x))
            [0xF, x, 0x6, 0x5] => todo!(), // load vx (load v0-vx to i through (i+x))
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn load() {
        let expected = [
            18, 78, 234, 172, 170, 234, 206, 170, 170, 174, 224, 160, 160, 224, 192, 64, 64, 224,
            224, 32, 192, 224, 224, 96, 32, 224, 160, 224, 32, 32, 96, 64, 32, 64, 224, 128, 224,
            224, 224, 32, 32, 32, 224, 224, 160, 224, 224, 224, 32, 224, 64, 160, 224, 160, 224,
            192, 128, 224, 224, 128, 192, 128, 160, 64, 160, 160, 162, 2, 218, 180, 0, 238, 162, 2,
            218, 180, 19, 220, 104, 1, 105, 5, 106, 10, 107, 1, 101, 42, 102, 43, 162, 22, 216,
            180, 162, 62, 217, 180, 162, 2, 54, 43, 162, 6, 218, 180, 107, 6, 162, 26, 216, 180,
            162, 62, 217, 180, 162, 6, 69, 42, 162, 2, 218, 180, 107, 11, 162, 30, 216, 180, 162,
            62, 217, 180, 162, 6, 85, 96, 162, 2, 218, 180, 107, 16, 162, 38, 216, 180, 162, 62,
            217, 180, 162, 6, 118, 255, 70, 42, 162, 2, 218, 180, 107, 21, 162, 46, 216, 180, 162,
            62, 217, 180, 162, 6, 149, 96, 162, 2, 218, 180, 107, 26, 162, 50, 216, 180, 162, 62,
            217, 180, 34, 66, 104, 23, 105, 27, 106, 32, 107, 1, 162, 10, 216, 180, 162, 54, 217,
            180, 162, 2, 218, 180, 107, 6, 162, 42, 216, 180, 162, 10, 217, 180, 162, 6, 135, 80,
            71, 42, 162, 2, 218, 180, 107, 11, 162, 42, 216, 180, 162, 14, 217, 180, 162, 6, 103,
            42, 135, 177, 71, 43, 162, 2, 218, 180, 107, 16, 162, 42, 216, 180, 162, 18, 217, 180,
            162, 6, 102, 120, 103, 31, 135, 98, 71, 24, 162, 2, 218, 180, 107, 21, 162, 42, 216,
            180, 162, 22, 217, 180, 162, 6, 102, 120, 103, 31, 135, 99, 71, 103, 162, 2, 218, 180,
            107, 26, 162, 42, 216, 180, 162, 26, 217, 180, 162, 6, 102, 140, 103, 140, 135, 100,
            71, 24, 162, 2, 218, 180, 104, 44, 105, 48, 106, 52, 107, 1, 162, 42, 216, 180, 162,
            30, 217, 180, 162, 6, 102, 140, 103, 120, 135, 101, 71, 236, 162, 2, 218, 180, 107, 6,
            162, 42, 216, 180, 162, 34, 217, 180, 162, 6, 102, 224, 134, 110, 70, 192, 162, 2, 218,
            180, 107, 11, 162, 42, 216, 180, 162, 54, 217, 180, 162, 6, 102, 15, 134, 102, 70, 7,
            162, 2, 218, 180, 107, 16, 162, 58, 216, 180, 162, 30, 217, 180, 163, 232, 96, 0, 97,
            48, 241, 85, 163, 233, 240, 101, 162, 6, 64, 48, 162, 2, 218, 180, 107, 21, 162, 58,
            216, 180, 162, 22, 217, 180, 163, 232, 102, 137, 246, 51, 242, 101, 162, 2, 48, 1, 162,
            6, 49, 3, 162, 6, 50, 7, 162, 6, 218, 180, 107, 26, 162, 14, 216, 180, 162, 62, 217,
            180, 18, 72, 19, 220,
        ];
        let mut cpu = CHIP8::default();
        let path = Path::new("./resources/test_opcode.ch8");
        let file = cpu.load_from_file(&path);
        let range = PROGRAM_MEMORY_START..PROGRAM_MEMORY_START + expected.len();
        assert_eq!(expected, cpu.memory[range])
    }

    #[test]
    fn test_load_clear_screen() {
        let program = [0x00, 0xE0];
        let mut cpu = CHIP8::default();
        cpu.pc = 0x200;
        cpu.load_from_slice(&program);
        cpu.execute();
        assert_eq!(cpu.pc, 0x202);
    }
}
