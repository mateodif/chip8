#[derive(Debug)]
pub struct Registers(pub [u8; crate::chip8::REGISTER_SIZE]);
impl std::ops::Index<u16> for Registers {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl std::ops::Index<u8> for Registers {
    type Output = u8;
    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl std::ops::IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}
