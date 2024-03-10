use crate::Uc8151;
use bitvec::array::BitArray;

#[derive(Default)]
pub struct Framebuffer {
    bits: BitArray<[u8; (Self::HEIGHT * Self::WIDTH) / 8]>,
}

impl Framebuffer {
    const HEIGHT: usize = 128;
    const WIDTH: usize = 297;

    pub fn write(&mut self, x: usize, y: usize, value: bool) {}

    pub fn real(&self, x: usize, y: usize) -> Option<bool> {
        None
    }
}

// TODO: implement
// https://docs.rs/embedded-graphics-core/latest/embedded_graphics_core/draw_target/trait.DrawTarget.html
pub struct Display<'a> {
    pub uc8151: Uc8151<'a>,
    pub framebuffer: Framebuffer,
}
