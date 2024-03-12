use crate::Uc8151;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Dimensions, OriginDimensions, Size},
    pixelcolor::BinaryColor,
    Pixel,
};

impl Default for Framebuffer {
    fn default() -> Self {
        Self {
            bits: [0; (Self::HEIGHT * Self::WIDTH) as usize / 8],
        }
    }
}

pub struct Framebuffer {
    bits: [u8; (Self::HEIGHT * Self::WIDTH) as usize / 8],
}

impl Framebuffer {
    pub const HEIGHT: usize = 128;
    pub const WIDTH: usize = 297;

    pub fn write(&mut self, x: usize, y: usize, value: bool) {
        if x >= Framebuffer::WIDTH || y >= Framebuffer::HEIGHT {
            // TODO: error out
            log::error!("Ignoring out of bounds pixel draw at x: {x} y:{y}");
            return;
        }

        let address = ((y / 8) + (x * (Self::HEIGHT / 8))) as usize;

        let o: u8 = 7 - (y as u8 & 0b111); // bit offset within byte
        let m: u8 = !(1 << o); // bit mask for byte
        let b: u8 = ((value) as u8) << o; // bit value shifted to position
        self.bits[address] = (self.bits[address] & m) | b;
    }

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

impl<'a> Display<'a> {
    /// Write current framebuffer to display and refresh
    pub async fn push_to_display(&mut self) {
        self.uc8151.update(&self.framebuffer.bits).await;
    }

    /// Clear framebuffer - call [Self::push_to_display] to clear display.
    pub async fn clear_buffer(&mut self) {
        self.framebuffer.bits.fill(0);
    }
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
                self.framebuffer
                    .write(pos.x as _, pos.y as _, color == BinaryColor::Off);
            });

        Ok(())
    }
}
impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        Size::new(Framebuffer::WIDTH as u32, Framebuffer::HEIGHT as u32)
    }
}
