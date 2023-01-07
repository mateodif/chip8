#![allow(dead_code)]
#![allow(unused_variables)]
use rand::Rng;
use std::default::Default;
use std::fs::read;
use std::path::Path;

pub const MEMORY_SIZE: usize = 4 * 1024; // 0x1000 directions, from 0x0 to 0xFFF.
pub const DISPLAY_SIZE: usize = 64 * 32;
pub const REGISTER_SIZE: usize = 16;
pub const PROGRAM_MEMORY_START: usize = 0x200; // Programs usually start a 0x200.

#[inline]
fn high_nibble(b: u8) -> u8 {
    (b >> 4) & 0x0F
}

#[inline]
fn low_nibble(b: u8) -> u8 {
    b & 0x0F
}

#[inline]
fn low_and_high_nibbles(b: u8) -> [u8; 2] {
    [high_nibble(b), low_nibble(b)]
}

#[inline]
fn from_low_and_high(a: u8, b: u8) -> u8 {
    a << 4 | b
}

#[inline]
fn address_from_nibbles(a: u8, b: u8, c: u8) -> u16 {
    ((a as u16) << 8) + ((b as u16) << 4) + (c as u16)
}

#[inline]
fn least_significant_bit(a: u8) -> u8 {
    a & 1
}

#[inline]
fn most_significant_bit(a: u8) -> u8 {
    1 << (a - 1)
}

#[derive(Debug)]
pub struct CHIP8 {
    memory: [u8; MEMORY_SIZE],
    display: [u8; DISPLAY_SIZE],
    registers: [u8; REGISTER_SIZE],
    stack: Vec<u16>,
    pc: u16,
    sp: u8,
    index: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Default for CHIP8 {
    fn default() -> CHIP8 {
        CHIP8 {
            memory: [0u8; MEMORY_SIZE],
            display: [0u8; DISPLAY_SIZE],
            registers: [0u8; REGISTER_SIZE],
            stack: Vec::new(),
            pc: 0x0,
            sp: 0x0,
            index: 0x0,
            delay_timer: 0x0,
            sound_timer: 0x0,
        }
    }
}

impl CHIP8 {
    // Instructions

    fn clear_display(&mut self) {
        self.display = [0u8; DISPLAY_SIZE];
    }

    fn return_from_routine(&mut self) {
        self.pc = self
            .stack
            .pop()
            .expect("Trying to return from non-existent routine");
        self.pc -= 1;
    }

    fn jump(&mut self, address: u16) {
        self.pc = address;
    }

    fn call_sub_routine(&mut self, address: u16) {
        self.sp += 1;
        self.stack.push(self.pc);
        self.pc = address;
    }

    fn skip_next_eq(&mut self, x: u8, val: u8) {
        // if vx == kk then
        if self.registers[x as usize] == val {
            self.pc += 2
        }
    }

    fn skip_next_not_eq(&mut self, x: u8, val: u8) {
        // if vx != kk then
        if self.registers[x as usize] != val {
            self.pc += 2
        }
    }

    fn skip_next_eq_reg(&mut self, x: u8, y: u8) {
        // if vx == vy then
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2
        }
    }

    fn assign_reg_to_immediate(&mut self, r: u8, n: u8) {
        self.registers[r as usize] = n;
    }

    fn sum_reg_with_immediate(&mut self, r: u8, n: u8) {
        self.registers[r as usize] += n;
    }

    fn assign_reg(&mut self, r1: u8, r2: u8) {
        self.registers[r1 as usize] = self.registers[r2 as usize]
    }

    fn bitor_assign(&mut self, r1: u8, r2: u8) {
        self.registers[r1 as usize] |= self.registers[r2 as usize]
    }

    fn bitand_assign(&mut self, r1: u8, r2: u8) {
        self.registers[r1 as usize] &= self.registers[r2 as usize]
    }

    fn bitxor_assign(&mut self, r1: u8, r2: u8) {
        self.registers[r1 as usize] &= self.registers[r2 as usize]
    }

    fn bitadd_assign(&mut self, r1: u8, r2: u8) {
        // The values of Vx and Vy are added together.
        // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
        // Only the lowest 8 bits of the result are kept, and stored in Vx.
        let (val, curr_carry) = (self.registers[r1 as usize], self.registers[0xF]);
        let (sum, new_carry) = val.carrying_add(self.registers[r2 as usize], curr_carry != 0);
        self.registers[r1 as usize] = sum;
        self.registers[0xF] = new_carry as u8;
    }

    fn bitsub_assign(&mut self, r1: u8, r2: u8) {
        // If Vx > Vy, then VF is set to 1, otherwise 0.
        // Then Vy is subtracted from Vx, and the results stored in Vx.
        let (val, curr_borrow) = (self.registers[r1 as usize], self.registers[0xF]);
        let (sub, new_borrow) = val.borrowing_sub(self.registers[r2 as usize], curr_borrow != 0);
        self.registers[r1 as usize] = sub;
        self.registers[0xF] = new_borrow as u8;
    }

    fn bitshr_assign(&mut self, r1: u8, r2: u8) {
        // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
        // Then Vx is divided by 2.
        if least_significant_bit(self.registers[r1 as usize]) == 1 {
            self.registers[0xF] = 1;
        }
        self.registers[r1 as usize] >>= self.registers[r2 as usize];
    }

    fn bitsubn_assign(&mut self, r1: u8, r2: u8) {
        // If Vy > Vx, then VF is set to 1, otherwise 0.
        // Then Vx is subtracted from Vy, and the results stored in Vx.
        let (val, curr_borrow) = (self.registers[r2 as usize], self.registers[0xF]);
        let (sub, new_borrow) = val.borrowing_sub(self.registers[r1 as usize], curr_borrow != 0);
        self.registers[r1 as usize] = sub;
        self.registers[0xF] = (!new_borrow) as u8;
    }

    fn bitshl_assign(&mut self, r1: u8, r2: u8) {
        // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
        // Then Vx is multiplied by 2.
        if most_significant_bit(self.registers[r1 as usize]) == 1 {
            self.registers[0xF] = 1;
        }
        self.registers[r1 as usize] <<= self.registers[r2 as usize];
    }

    fn not_eq_regs(&mut self, x: u8, y: u8) {
        // if vx != vy then
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
    }

    fn assign_index_to(&mut self, address: u16) {
        self.index = address;
    }

    fn jump_with_v0(&mut self, address: u16) {
        self.jump(address + (self.registers[0] as u16))
    }

    fn assign_reg_randomly_with_and(&mut self, x: u8, val: u8) {
        self.registers[x as usize] = rand::thread_rng().gen::<u8>() & val;
    }

    // Internals

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
        match instruction {
            [0x0, 0x0, 0xE, 0x0] => self.clear_display(),
            [0x0, 0x0, 0xE, 0xE] => self.return_from_routine(),
            [0x1, n1, n2, n3] => self.jump(address_from_nibbles(n1, n2, n3)),
            [0x2, n1, n2, n3] => self.call_sub_routine(address_from_nibbles(n1, n2, n3)),
            [0x3, x, k1, k2] => self.skip_next_eq(x, from_low_and_high(k1, k2)),
            [0x4, x, k1, k2] => self.skip_next_not_eq(x, from_low_and_high(k1, k2)),
            [0x5, x, y, 0x0] => self.skip_next_eq_reg(x, y),
            [0x6, x, k1, k2] => self.assign_reg_to_immediate(x, from_low_and_high(k1, k2)),
            [0x7, x, k1, k2] => self.sum_reg_with_immediate(x, from_low_and_high(k1, k2)),
            [0x8, x, y, 0x0] => self.assign_reg(x, y),
            [0x8, x, y, 0x1] => self.bitor_assign(x, y),
            [0x8, x, y, 0x2] => self.bitand_assign(x, y),
            [0x8, x, y, 0x3] => self.bitxor_assign(x, y),
            [0x8, x, y, 0x4] => self.bitadd_assign(x, y),
            [0x8, x, y, 0x5] => self.bitsub_assign(x, y),
            [0x8, x, y, 0x6] => self.bitshr_assign(x, y),
            [0x8, x, y, 0x7] => self.bitsubn_assign(x, y),
            [0x8, x, y, 0xE] => self.bitshl_assign(x, y),
            [0x9, x, y, 0x0] => self.not_eq_regs(x, y),
            [0xA, n1, n2, n3] => self.assign_index_to(address_from_nibbles(n1, n2, n3)),
            [0xB, n1, n2, n3] => self.jump_with_v0(address_from_nibbles(n1, n2, n3)),
            [0xC, x, k1, k2] => self.assign_reg_randomly_with_and(x, from_low_and_high(k1, k2)),
            [0xD, x, y, n] => todo!(), // sprite vx vy N (vf = 1 on collision)
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
    use itertools::Itertools;
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

    #[test]
    fn test_set_from_register() {
        let mut cpu = CHIP8::default();
        for reg_x in 0x0..=0xF {
            for reg_y in 0x0..=0xF {
                cpu.pc = 0x200;
                // The "program" is the LD instruction.
                let program = [0x80 + reg_x, reg_y << 4];
                cpu.load_from_slice(&program);
                cpu.execute();
                assert_eq!(cpu.pc, 0x202);
                assert_eq!(cpu.registers[reg_x as usize], cpu.registers[reg_y as usize]);
            }
        }
    }
    #[test]
    fn test_set_register_to_immediate() {
        let mut cpu = CHIP8::default();
        for reg_x in 0x0..=0xF {
            for nibbles in (0x0..=0xF).permutations(2).collect_vec() {
                cpu.pc = 0x200;
                let [nibble_1, nibble_2] = &nibbles[..] else {panic!("Permutations are working weirdly")};
                let expected_val = from_low_and_high(*nibble_1, *nibble_2);
                let program = [0x60 + reg_x, expected_val];
                cpu.load_from_slice(&program);
                cpu.execute();
                assert_eq!(cpu.pc, 0x202);
                assert_eq!(cpu.registers[reg_x as usize], expected_val);
            }
        }
    }

    #[test]
    fn test_sum_register_with_immediate() {
        let mut cpu = CHIP8::default();
        for reg_x in 0x0..=0xF {
            for nibbles in (0x0..=0xF).permutations(2).collect_vec() {
                cpu.pc = 0x200;
                let [nibble_1, nibble_2] = &nibbles[..] else {panic!("Permutations are working weirdly")};
                let expected_val = cpu.registers[reg_x as usize]
                    .wrapping_add(from_low_and_high(*nibble_1, *nibble_2));
                let program = [0x60 + reg_x, expected_val];
                cpu.load_from_slice(&program);
                cpu.execute();
                assert_eq!(cpu.pc, 0x202);
                assert_eq!(cpu.registers[reg_x as usize], expected_val);
            }
        }
    }
    #[test]
    fn test_jump_immediate() {
        let program = [0x12, 0x22];
        let mut cpu = CHIP8::default();
        cpu.pc = 0x200;
        cpu.load_from_slice(&program);
        cpu.execute();
        assert_eq!(cpu.pc, 0x222);
    }
    #[test]
    fn test_call_subroutine() {
        let mut cpu = CHIP8::default();
        for nibbles in (0x0..=0xF).permutations(3).collect_vec() {
            let [nibble_1, nibble_2, nibble_3] = &nibbles[..] else {panic!("Permutations are working weirdly")};
            println!("{:?}", nibbles);
            let call_instruction = [0x20 + nibble_1, (nibble_2 << 4) + nibble_3];
            cpu.pc = 0x200;
            cpu.load_from_slice(&call_instruction);
            cpu.execute();
            assert_eq!(
                cpu.pc,
                address_from_nibbles(*nibble_1, *nibble_2, *nibble_3)
            );
            cpu.sp = 0;
        }
    }
}
