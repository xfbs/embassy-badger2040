use embassy_rp::{
    gpio::{Input, Pull},
    peripherals::*,
};
use embassy_time::Timer;

pub struct Buttons<'a> {
    pub a: Input<'a>,
    pub b: Input<'a>,
    pub c: Input<'a>,
    pub d: Input<'a>,
    pub e: Input<'a>,
}

impl<'a> Buttons<'a> {
    pub fn new(e: PIN_11, a: PIN_12, b: PIN_13, c: PIN_14, d: PIN_15) -> Self {
        Self {
            a: Input::new(a, Pull::Down),
            b: Input::new(b, Pull::Down),
            c: Input::new(c, Pull::Down),
            d: Input::new(d, Pull::Down),
            e: Input::new(e, Pull::Down),
        }
    }
}

