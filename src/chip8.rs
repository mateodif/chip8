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
fn from_nibbles(a: u8, b: u8, c: u8, d: u8) -> u16 {
    u16::from_be_bytes([from_low_and_high(a, b), from_low_and_high(c, d)])
}

#[inline]
fn address_from_nibbles(a: u8, b: u8, c: u8) -> u16 {
    ((a as u16) << 8) + ((b as u16) << 4) + (c as u16)
}

#[derive(Debug)]
enum Instruction {
    NoOperation,
    ClearScreen,
    ReturnFromSubroutine,
    Jump { address: u16 },
    CallSubroutine { address: u16 },
    SkipIfEqual { register: u8, byte: u8 },
    SkipIfNotEqual { register: u8, byte: u8 },
    SkipIfRegisterEqual { register1: u8, register2: u8 },
    LoadByteIntoRegister { register: u8, byte: u8 },
    AddByteToRegister { register: u8, byte: u8 },
    LoadRegisterIntoRegister { register1: u8, register2: u8 },
    OrRegisters { register1: u8, register2: u8 },
    AndRegisters { register1: u8, register2: u8 },
    XorRegisters { register1: u8, register2: u8 },
    AddRegisters { register1: u8, register2: u8 },
    SubRegisters { register1: u8, register2: u8 },
    ShiftRight { register1: u8, register2: u8 },
    SubNRegisters { register1: u8, register2: u8 },
    ShiftLeft { register1: u8, register2: u8 },
    SkipIfRegisterNotEqual { register1: u8, register2: u8 },
    LoadAddressIntoIndex { address: u16 },
    JumpToAddressPlusV0 { address: u16 },
    RandomByteAndIntoRegister { register: u8, byte: u8 },
    DrawSprite { register1: u8, register2: u8, nibble: u8 },
    SkipIfKeyPressed { register: u8 },
    SkipIfKeyNotPressed { register: u8 },
    LoadDelayTimerIntoRegister { register: u8 },
    WaitForKeyPress { register: u8 },
    LoadRegisterIntoDelayTimer { register: u8 },
    LoadRegisterIntoSoundTimer { register: u8 },
    AddRegisterToIndex { register: u8 },
    LoadFontLocationIntoIndex { register: u8 },
    LoadBinaryCodedDecimalIntoMemory { register: u8 },
    LoadRegistersIntoMemory { register: u8 },
    LoadMemoryIntoRegisters { register: u8 },
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

    // Load program from address, if not specified,
    // default to 0x200.
    pub fn load_from_slice(&mut self, slice: &[u8], address: Option<u16>) {
        let start_address = match address {
            Some(address) => address,
            None => PROGRAM_MEMORY_START as u16,
        };
        for (i, _) in slice.iter().enumerate() {
            self.memory[i + start_address as usize] = slice[i];
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
        self.registers[r as usize] = n;
    }

    fn sum_register_with_immediate(&mut self, r: u8, n: u8) {
        self.registers[r as usize] += n;
    }

    fn operate_and_set_register(&mut self, destination: u8, source: u8, op: fn(u8, u8) -> u8) {
        self.registers[destination as usize] = op(self.registers[destination as usize], self.registers[source as usize]);
    }

    fn return_from_sub_routine(&mut self) {
        self.pc = self.stack.pop().unwrap();
        self.sp -= 1;
    }
    fn call_sub_routine(&mut self, address: u16) {
        self.sp += 1;
        self.stack.push(self.pc);
        self.pc = address;
    }

    fn fetch(&mut self) -> Instruction {
        let upc = self.pc as usize;
        let [first_nibble, second_nibble] = low_and_high_nibbles(self.memory[upc]);
        let [third_nibble, fourth_nibble] = low_and_high_nibbles(self.memory[upc + 1]);
        self.pc += 2;
        [first_nibble, second_nibble, third_nibble, fourth_nibble];

        match [first_nibble, second_nibble, third_nibble, fourth_nibble] {
            [0x0, 0x0, 0xE, 0x0] => Instruction::ClearScreen,
            [0x0, 0x0, 0xE, 0xE] => Instruction::ReturnFromSubroutine,
            [0x1, n1, n2, n3] => Instruction::Jump { address: from_nibbles(0x0, n1, n2, n3) },
            [0x2, n1, n2, n3] => Instruction::CallSubroutine { address: from_nibbles(0x0, n1, n2, n3) },
            [0x3, x, n1, n2] => Instruction::SkipIfEqual { register: x, byte: from_low_and_high(n1, n2) },
            [0x4, x, n1, n2] => Instruction::SkipIfNotEqual { register: x, byte: from_low_and_high(n1, n2) },
            [0x5, x, y, 0x0] => Instruction::SkipIfRegisterEqual { register1: x, register2: y },
            [0x6, x, n1, n2] => Instruction::LoadByteIntoRegister { register: x, byte: from_low_and_high(n1, n2) },
            [0x7, x, n1, n2] => Instruction::AddByteToRegister { register: x, byte: from_low_and_high(n1, n2) },
            [0x8, x, y, 0x0] => Instruction::LoadRegisterIntoRegister { register1: x, register2: y },
            [0x8, x, y, 0x1] => Instruction::OrRegisters { register1: x, register2: y },
            [0x8, x, y, 0x2] => Instruction::AndRegisters { register1: x, register2: y },
            [0x8, x, y, 0x3] => Instruction::XorRegisters { register1: x, register2: y },
            [0x8, x, y, 0x4] => Instruction::AddRegisters { register1: x, register2: y },
            [0x8, x, y, 0x5] => Instruction::SubRegisters { register1: x, register2: y },
            [0x8, x, y, 0x6] => Instruction::ShiftRight { register1: x, register2: y },
            [0x8, x, y, 0x7] => Instruction::SubNRegisters { register1: x, register2: y },
            [0x8, x, y, 0xE] => Instruction::ShiftLeft { register1: x, register2: y },
            [0x9, x, y, 0x0] => Instruction::SkipIfRegisterNotEqual { register1: x, register2: y },
            [0xA, n1, n2, n3] => Instruction::LoadAddressIntoIndex { address: address_from_nibbles(n1, n2, n3) },
            [0xB, n1, n2, n3] => Instruction::JumpToAddressPlusV0 { address: address_from_nibbles(n1, n2, n3) },
            [0xC, x, n2, n3] => Instruction::RandomByteAndIntoRegister { register: x, byte: from_low_and_high(n2, n3) },
            [0xD, x, y, n] => Instruction::DrawSprite { register1: x, register2: y, nibble: n },
            [0xE, x, 0x9, 0xE] => Instruction::SkipIfKeyPressed { register: x },
            [0xE, x, 0xA, 0x1] => Instruction::SkipIfKeyNotPressed { register: x },
            [0xF, x, 0x0, 0x7] => Instruction::LoadDelayTimerIntoRegister { register: x },
            [0xF, x, 0x0, 0xA] => Instruction::WaitForKeyPress { register: x },
            [0xF, x, 0x1, 0x5] => Instruction::LoadRegisterIntoDelayTimer { register: x },
            [0xF, x, 0x1, 0x8] => Instruction::LoadRegisterIntoSoundTimer { register: x },
            [0xF, x, 0x1, 0xE] => Instruction::AddRegisterToIndex { register: x },
            [0xF, x, 0x2, 0x9] => Instruction::LoadFontLocationIntoIndex { register: x },
            [0xF, x, 0x3, 0x3] => Instruction::LoadBinaryCodedDecimalIntoMemory { register: x },
            [0xF, x, 0x5, 0x5] => Instruction::LoadRegistersIntoMemory { register: x },
            [0xF, x, 0x6, 0x5] => Instruction::LoadMemoryIntoRegisters { register: x },
            _ => panic!("Unknown instruction"),
        }

    }

    pub fn execute(&mut self) {
        let instruction = self.fetch();
        println!("Fetched instruction: {:?}", instruction);
        match instruction {
            Instruction::ClearScreen => self.clear_screen(),
            Instruction::ReturnFromSubroutine => self.return_from_sub_routine(),
            Instruction::Jump { address } => {
                self.pc = address;
            },
            Instruction::CallSubroutine { address } => {
                self.call_sub_routine(address);
            },
            Instruction::SkipIfEqual { register, byte } => todo!(),
            Instruction::SkipIfNotEqual { register, byte } => todo!(),
            Instruction::SkipIfRegisterEqual { register1, register2 } => todo!(),
            Instruction::LoadByteIntoRegister { register, byte } => {
                self.set_register_to_immediate(register, byte);
            },
            Instruction::AddByteToRegister { register, byte } => {
                self.sum_register_with_immediate(register, byte);
            },
            Instruction::LoadRegisterIntoRegister { register1, register2 } => {
                self.operate_and_set_register(register1, register2, |register1, register2| register2);
            },
            Instruction::OrRegisters { register1, register2 } => {
                self.operate_and_set_register(register1, register2, u8::bitor);
            },
            Instruction::AndRegisters { register1, register2 } => {
                self.operate_and_set_register(register1, register2, u8::bitand);
            },
            Instruction::XorRegisters { register1, register2 } => {
                self.operate_and_set_register(register1, register2, u8::bitxor);
            },
            Instruction::AddRegisters { register1, register2 } => {
                // TODO: set carry
                self.operate_and_set_register(register1, register2, u8::add);
            },
            Instruction::SubRegisters { register1, register2 } => {
                // TODO: set borrow
                self.operate_and_set_register(register1, register2, u8::sub);
            },
            Instruction::ShiftRight { register1, register2 } => {
                // TODO: If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0
                // THIS IS LIKELY NOT CORRECT!
                self.operate_and_set_register(register1, register2, u8::shr);
            },
            Instruction::SubNRegisters { register1, register2 } => {
                // TODO: set borrow
                let f = |register1: u8, register2: u8| -> u8 { u8::sub(register1, register2) };
                self.operate_and_set_register(register1, register2, f);
            },
            Instruction::ShiftLeft { register1, register2 } => {
                // THIS IS LIKELY NOT CORRECT!
                self.operate_and_set_register(register1, register2, u8::shl);
            },
            Instruction::SkipIfRegisterNotEqual { register1, register2 } => todo!(),
            Instruction::LoadAddressIntoIndex { address } => todo!(),
            Instruction::JumpToAddressPlusV0 { address } => todo!(),
            Instruction::RandomByteAndIntoRegister { register, byte } => todo!(),
            Instruction::DrawSprite { register1, register2, nibble } => todo!(),
            Instruction::SkipIfKeyPressed { register } => todo!(),
            Instruction::SkipIfKeyNotPressed { register } => todo!(),
            Instruction::LoadDelayTimerIntoRegister { register } => todo!(),
            Instruction::WaitForKeyPress { register } => todo!(),
            Instruction::LoadRegisterIntoDelayTimer { register } => todo!(),
            Instruction::LoadRegisterIntoSoundTimer { register } => todo!(),
            Instruction::AddRegisterToIndex { register } => todo!(),
            Instruction::LoadFontLocationIntoIndex { register } => todo!(),
            Instruction::LoadBinaryCodedDecimalIntoMemory { register } => todo!(),
            Instruction::LoadRegistersIntoMemory { register } => todo!(),
            Instruction::LoadMemoryIntoRegisters { register } => todo!(),
            _ => panic!("Unknown instruction"),
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
        cpu.load_from_slice(&program, None);
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
                cpu.load_from_slice(&program, None);
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
                cpu.load_from_slice(&program, None);
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
                cpu.load_from_slice(&program, None);
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
        cpu.load_from_slice(&program, None);
        cpu.execute();
        assert_eq!(cpu.pc, 0x222);
    }
    #[test]
    fn test_call_and_return_subroutine() {
        let mut cpu = CHIP8::default();
        for nibbles in (0x0..=0xF).permutations(3).collect_vec() {
            // First execute a call, and then return
            // to the original position using return.
            let [nibble_1, nibble_2, nibble_3] = &nibbles[..] else {panic!("Permutations are working weirdly")};
            let call_instruction = [0x20 + nibble_1, (nibble_2 << 4) + nibble_3];
            cpu.pc = 0x200;
            cpu.load_from_slice(&call_instruction, None);
            cpu.execute();
            assert_eq!(
                cpu.pc,
                address_from_nibbles(*nibble_1, *nibble_2, *nibble_3)
            );
            let ret_instruction = [0x0, 0xEE];
            cpu.load_from_slice(&ret_instruction, Some(address_from_nibbles(*nibble_1, *nibble_2, *nibble_3)));
            cpu.execute();
            assert_eq!(cpu.pc, 0x202);
            cpu.sp = 0;
        }
    }
}
