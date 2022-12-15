const MEMORY_SIZE: usize = 4 * 1024;
const DISPLAY_SIZE: usize = 64 * 32;
const REGISTER_SIZE: usize = 16;

fn high_nibble(b: u8) -> u8 {
    (b >> 4) & 0x0F
}

fn low_nibble(b: u8) -> u8 {
    b & 0x0F
}

fn low_and_high_nibbles(b: u8) -> [u8; 2] {
    [low_nibble(b), high_nibble(b)]
}

fn to_nibbles(b: u16) -> [u8; 4] {
    let [x, y] = b.to_be_bytes();
    [low_nibble(x), high_nibble(x), low_nibble(y), high_nibble(y)]
}

#[derive(Debug)]
struct CHIP8 {
    memory: [u8; MEMORY_SIZE],
    display: [u8; DISPLAY_SIZE],
    register: [u8; REGISTER_SIZE],
    stack: Vec<u16>,
    pc: u16,
    index: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl CHIP8 {
    fn load_font(&mut self) {
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

    fn load_testing_data(&mut self) {
        self.memory[0x200] = 0x00;
        self.memory[0x201] = 0xE0;
        self.pc = 0x200;
    }

    fn clear_screen(&mut self) {
        self.display = [0u8; DISPLAY_SIZE];
    }

    fn fetch(&mut self) -> [u8; 4] {
        let upc = self.pc as usize;
        let [fi, se] = low_and_high_nibbles(self.memory[upc]);
        let [th, fo] = low_and_high_nibbles(self.memory[upc+1]);
        self.pc += 2;
        [fi, se, th, fo]
    }

    fn execute(&mut self) {
        let instruction = self.fetch();
        println!("{} {} {} {}", instruction[0], instruction[1], instruction[2], instruction[3]);
        match instruction {
            [_, _, _, _] => self.clear_screen(),
            _ => todo!(),
        }
    }
}

fn main() {
    let mut chip = CHIP8 {
        memory: [0u8; MEMORY_SIZE],
        display: [0u8; DISPLAY_SIZE],
        register: [0u8; REGISTER_SIZE],
        stack: Vec::new(),
        pc: 0x0,
        index: 0x0,
        delay_timer: 0x0,
        sound_timer: 0x0,
    };

    chip.load_font();

    chip.load_testing_data();

    chip.execute();

}
