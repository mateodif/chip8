pub mod chip8;
use crate::chip8::CHIP8;
fn main() {
    let mut chip = CHIP8::default();
    chip.load_font();

    chip.load_testing_data();

    chip.execute();

}
