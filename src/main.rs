//! This example shows how to use USB (Universal Serial Bus) in the RP2040 chip.
//!
//! This creates the possibility to send log::info/warn/error/debug! to USB serial port.

#![no_std]
#![no_main]

use embassy_badger2040::{Buttons, Display, Framebuffer, Uc8151};
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Level, Output, Pull},
    peripherals::USB,
    spi::{self, Spi},
    usb::{Driver, InterruptHandler},
};
use embassy_time::Timer;
use embedded_graphics::{geometry::{Point, Size}, mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle}, pixelcolor::BinaryColor, primitives::{Primitive, PrimitiveStyle, Rectangle}, Drawable};
use embedded_text::{alignment::HorizontalAlignment, style::{HeightMode, TextBoxStyleBuilder}, TextBox};
use log::info;
use uc8151::WIDTH;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // create buttons
    let mut buttons = Buttons::new(p.PIN_11, p.PIN_12, p.PIN_13, p.PIN_14, p.PIN_15);

    let pin_user = p.PIN_23;
    //let pin_cs          = p.PIN_17;
    //let pin_clk         = p.PIN_18;
    //let pin_mosi        = p.PIN_19;
    //let pin_dc          = p.PIN_20;
    //let pin_reset       = p.PIN_21;
    //let pin_busy        = p.PIN_26;
    let pin_vbus_detect = p.PIN_24;
    let pin_led = p.PIN_25;
    let pin_battery = p.PIN_29;
    let pin_enable_3v3 = p.PIN_10;

    // create SPI
    /*
    let mut config = spi::Config::default();
    config.frequency = 12_000_000;
    let mut spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_16, config);
    let mut busy = Input::new(p.PIN_26, Pull::Up);
    let mut cs = Output::new(p.PIN_17, Level::High);
    let mut dc = Output::new(p.PIN_20, Level::Low);
    let mut reset = Output::new(p.PIN_21, Level::High);
    */

    // E-INK device init
    let mut uc8151 = Uc8151::new(
        p.SPI0, p.PIN_17, p.PIN_18, p.PIN_19, p.PIN_16, p.PIN_20, p.PIN_21, p.PIN_26,
    );

    uc8151.init().await;
    uc8151.update(&[0; (128 * 296) / 8]).await;
    // setup display (device + framebuffer)
    let mut display = Display {
        framebuffer: Default::default(),
        uc8151,
    };

    // flash LED + log output while waiting for button A press
    let mut light = false;
    let mut led = Output::new(pin_led, Level::Low);
    while buttons.a.is_low() {
        log::info!("Printing graphics if pressed again");
        match light {
            true => led.set_high(),
            false => led.set_low(),
        }
        light = !light;

        select(buttons.a.wait_for_rising_edge(),Timer::after_secs(1)).await;
    }
    led.set_low();

    // Create text
    let text = "Hi! I'm Aron.\nDon't talk to\nme about\nEmbedded Rust.";
    // Note we're setting the Text color to `Off`. The driver is set up to treat Off as Black so that BMPs work as expected.
    let character_style = MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::Off);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Center)
        .vertical_alignment(embedded_text::alignment::VerticalAlignment::Middle)
        .paragraph_spacing(6)
        .build();
    // Bounding box for our text. Fill it with the opposite color so we can read the text.
    let bounds = Rectangle::new(Point::new(157, 10), Size::new(WIDTH - 157, 0));
    bounds
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display) // draw to framebuffer
        .unwrap();
    // Create the text box and apply styling options.
    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);
    // Draw the text box.
    text_box.draw(&mut display).unwrap(); // draw to framebuffer
    // push framebuffer to display
    display.push_to_display().await;

    log::info!("Entering loop");

    let mut counter = 0;
    loop {
        buttons.a.wait_for_rising_edge().await;
        counter += 1;
        log::info!("Tick {}", counter);
        match led.is_set_high() {
            false => led.set_high(),
            true => led.set_low(),
        }
    }
}
