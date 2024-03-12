//! This example shows how to use USB (Universal Serial Bus) in the RP2040 chip.
//!
//! This creates the possibility to send log::info/warn/error/debug! to USB serial port.

#![no_std]
#![no_main]

use core::borrow::BorrowMut;
use core::cell::RefCell;

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_futures::select::{self, select};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::spi;
use embassy_rp::spi::{Blocking, Spi};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, Peripheral};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::{Mutex, NoopMutex};
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_hal_1::digital::OutputPin;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
use fugit::RateExtU32;
use static_cell::StaticCell;
use uc8151::WIDTH;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

const DISPLAY_FREQ: u32 = 64_000_000;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    log::info!("Press button 1 to continue");

    let mut button = Input::new(p.PIN_12, Pull::Down);

    while button.is_low() {
        log::info!("Press button A to init..");
        select(Timer::after_secs(1), button.wait_for_rising_edge()).await;
    }

    log::info!("Setting up display, fasten your seatbelts.");

    // display init start
    // let core = pac::CorePeripherals::take().unwrap();
    // // Grab our singleton objects
    // let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    // let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    // let clocks = hal::clocks::init_clocks_and_plls(
    //     pimoroni_badger2040::XOSC_CRYSTAL_FREQ,
    //     pac.XOSC,
    //     pac.CLOCKS,
    //     pac.PLL_SYS,
    //     pac.PLL_USB,
    //     &mut pac.RESETS,
    //     &mut watchdog,
    // )
    // .ok()
    // .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    // let sio = hal::Sio::new(pac.SIO);

    // // Set the pins up according to their function on this particular board
    // let pins = pimoroni_badger2040::Pins::new(
    //     pac.IO_BANK0,
    //     pac.PADS_BANK0,
    //     sio.gpio_bank0,
    //     &mut pac.RESETS,
    // );

    // Configure the timer peripheral to be a CountDown timer for our blinky delay
    // let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    // let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Get all the basic peripherals, and init clocks/timers
    // Enable 3.3V power or you won't see anything
    // let mut power = pins.p3v3_en.into_push_pull_output();
    // power.set_high().unwrap();
    let mut power = Output::new(p.PIN_10, Level::High);
    power.set_high();

    // let mut led_pin = pins.led.into_push_pull_output();

    // // TODO: use buttons somehow
    // let _buttons = Buttons {
    //     a: pins.sw_a.into_floating_input(),
    //     b: pins.sw_b.into_floating_input(),
    //     c: pins.sw_c.into_floating_input(),
    //     up: pins.sw_up.into_floating_input(),
    //     down: pins.sw_down.into_floating_input(),
    // };

    // Set up the pins for the e-ink display
    // let _spi_sclk = pins.sclk.into_mode::<hal::gpio::FunctionSpi>();
    // let _spi_mosi = pins.mosi.into_mode::<hal::gpio::FunctionSpi>();
    // let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);
    // let mut dc: Output<'_> = Output::new(p.PIN_20, Level::High);
    // let mut cs = p.PIN_17;//Output::new(p.PIN_17, Level::High);
    // let busy = pins.inky_busy.into_pull_up_input();
    // let reset = pins.inky_res.into_push_pull_output();

    let busy = Input::new(p.PIN_26, Pull::Up);
    let reset = Output::new(p.PIN_21, Level::High);

    // let spi = spi.init(
    //     &mut pac.RESETS,
    //     clocks.peripheral_clock.freq(),
    //     RateExtU32::MHz(1),
    //     &embedded_hal::spi::MODE_0,
    // );
    let clk = p.PIN_18;

    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;
    let spi: Spi<'_, _, Blocking> = Spi::new_blocking(
        p.SPI0,
        clk,
        p.PIN_19,
        unsafe { p.PIN_16.clone_unchecked() },
        display_config.clone(),
    );
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));
    // let display_spi = SpiDeviceWithConfig::new(&spi_bus, unsafe {p.PIN_17.clone_unchecked()}, display_config);

    static SPI_BUS: StaticCell<
        NoopMutex<RefCell<Spi<'_, embassy_rp::peripherals::SPI0, Blocking>>>,
    > = StaticCell::new();
    let spi_bus = SPI_BUS.init(spi_bus);
    let spi_cs_pin = Output::new(unsafe { p.PIN_17.clone_unchecked() }, Level::High);
    let spi_dev1 =
        embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice::new(spi_bus, spi_cs_pin);

    // dc.set_high();
    // cs.set_high();

    // let mut count_down = timer.count_down();
    let cs = Output::new(p.PIN_17, Level::High); //Output::new(p.PIN_17, Level::High);
    let dc = Output::new(p.PIN_20, Level::Low);
    let mut display = uc8151::Uc8151::new(spi_dev1, cs, dc, busy, reset);
    // display init end

    log::info!("Finished display setup noodles");
    display.disable();
    let mut delay = Delay;
    delay.delay_ms(10_u32);
    display.enable();
    while display.is_busy() {}

    log::info!("Drawing");

    // FIXME: is this valid for embassy ?
    // let core = pimoroni_badger2040::hal::pac::CorePeripherals::take().unwrap();
    // fixme: original is clocks.system_clock.freq().to_Hz()
    // let mut delay = cortex_m::delay::Delay::new(core.SYST, embassy_rp::clocks::clk_sys_freq());
    // Initialise display. Using the default LUT speed setting
    let _ = display.setup(&mut Delay, uc8151::LUT::Internal).unwrap();
    let text = "Hi! I'm Aron.\nDon't talk to\nme about\nEmbedded Rust.";
    // Note we're setting the Text color to `Off`. The driver is set up to treat Off as Black so that BMPs work as expected.
    let character_style = MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::Off);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Center)
        //.vertical_alignment(embedded_text::alignment::VerticalAlignment::Top)
        .paragraph_spacing(6)
        .build();
    // Bounding box for our text. Fill it with the opposite color so we can read the text.
    let bounds = Rectangle::new(Point::new(157, 10), Size::new(WIDTH - 157, 0));
    bounds
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();
    // Create the text box and apply styling options.
    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);
    // Draw the text box.
    text_box.draw(&mut display).unwrap();

    log::info!("Entering tick loop");

    let mut counter = 0;
    loop {
        counter += 1;
        log::info!("Tick {}", counter);

        Timer::after_secs(1).await;
    }
}
