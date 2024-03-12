//! This example shows how to use USB (Universal Serial Bus) in the RP2040 chip.
//!
//! This creates the possibility to send log::info/warn/error/debug! to USB serial port.

#![no_std]
#![no_main]

use embassy_badger2040::Display;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::Output,
    peripherals::USB,
    usb::{Driver, InterruptHandler},
};
use embassy_time::Timer;
use embedded_graphics::{
    geometry::{Point, Size},
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::BinaryColor,
    primitives::{Primitive, PrimitiveStyle, Rectangle},
    Drawable,
};
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
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

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        info!("LED OFF");
        led.set_low();
        Timer::after_secs(1).await;

        info!("LED ON");
        led.set_high();
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_badger2040::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();
    spawner.spawn(blink_task(p.LED)).unwrap();

    info!("Initialized");

    // setup display (device + framebuffer)
    let mut display = Display::new(p.UC8151).await;
    display.push_to_display().await;

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

    info!("Entering loop");

    // E-INK device init
    let mut button_a = p.BUTTON_A;
    let mut counter = 0;
    loop {
        button_a.wait_for_rising_edge().await;
        counter += 1;
        info!("Tick {}", counter);
    }
}
