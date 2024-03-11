use crate::Uc8151;
use bitvec::array::BitArray;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Dimensions, OriginDimensions, Size},
    pixelcolor::BinaryColor,
    Pixel,
};

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


impl<'a> DrawTarget for Display<'a> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;
    // adapted from https://github.com/9names/uc8151-rs
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| {
                if pos.x >= (Framebuffer::WIDTH as i32) || pos.y >= (Framebuffer::HEIGHT as i32) {
                    // TODO: error out
                    return;
                }
                self.framebuffer.write(pos.x as _, pos.y as _, color == BinaryColor::Off);
            });

        Ok(())
    }
}
impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        Size::new(Framebuffer::WIDTH as u32, Framebuffer::HEIGHT as u32)
    }
}
