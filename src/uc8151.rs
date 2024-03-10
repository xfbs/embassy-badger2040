use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    peripherals::*,
    spi::{self, Blocking, Spi},
    Peripherals,
};
use embassy_time::Timer;
use bitflags::bitflags;

pub struct Uc8151<'a> {
    spi: Spi<'a, SPI0, Blocking>,
    busy: Input<'a>,
    chip_select: Output<'a>,
    dc: Output<'a>,
    reset: Output<'a>,
}

pub enum Register {
    PSR = 0x00,
    PWR = 0x01,
    POF = 0x02,
    PFS = 0x03,
    PON = 0x04,
    PMES = 0x05,
    BTST = 0x06,
    DSLP = 0x07,
    DTM1 = 0x10,
    DSP = 0x11,
    DRF = 0x12,
    DTM2 = 0x13,
    LUT_VCOM = 0x20,
    LUT_WW = 0x21,
    LUT_BW = 0x22,
    LUT_WB = 0x23,
    LUT_BB = 0x24,
    PLL = 0x30,
    TSC = 0x40,
    TSE = 0x41,
    TSR = 0x43,
    TSW = 0x42,
    CDI = 0x50,
    LPD = 0x51,
    TCON = 0x60,
    TRES = 0x61,
    REV = 0x70,
    FLG = 0x71,
    AMV = 0x80,
    VV = 0x81,
    VDCS = 0x82,
    PTL = 0x90,
    PTIN = 0x91,
    PTOU = 0x92,
    PGM = 0xa0,
    APG = 0xa1,
    ROTP = 0xa2,
    CCSET = 0xe0,
    PWS = 0xe3,
    TSSET = 0xe5,
}

bitflags! {
    #[repr(transparent)]
    pub struct PsrFlags: u8 {
        const RES_96x230   = 0b00000000;
        const RES_96x252   = 0b01000000;
        const RES_128x296  = 0b10000000;
        const RES_160x296  = 0b11000000;

        const LUT_OTP      = 0b00000000;
        const LUT_REG      = 0b00100000;

        const FORMAT_BWR   = 0b00000000;
        const FORMAT_BW    = 0b00010000;

        const SCAN_DOWN    = 0b00000000;
        const SCAN_UP      = 0b00001000;

        const SHIFT_LEFT   = 0b00000000;
        const SHIFT_RIGHT  = 0b00000100;

        const BOOSTER_OFF  = 0b00000000;
        const BOOSTER_ON   = 0b00000010;

        const RESET_SOFT   = 0b00000000;
        const RESET_NONE   = 0b00000001;
    }

    #[repr(transparent)]
    pub struct PwrFlags1: u8 {
        const VDS_EXTERNAL = 0b00000000;
        const VDS_INTERNAL = 0b00000010;

        const VDG_EXTERNAL = 0b00000000;
        const VDG_INTERNAL = 0b00000001;
    }

    #[repr(transparent)]
    pub struct PwrFlags2: u8 {
        const VCOM_VD      = 0b00000000;
        const VCOM_VG      = 0b00000100;

        const VGHL_16V     = 0b00000000;
        const VGHL_15V     = 0b00000001;
        const VGHL_14V     = 0b00000010;
        const VGHL_13V     = 0b00000011;
    }

    #[repr(transparent)]
    pub struct BoosterFlags: u8 {
        const START_10MS = 0b00000000;
        const START_20MS = 0b01000000;
        const START_30MS = 0b10000000;
        const START_40MS = 0b11000000;

        const STRENGTH_1 = 0b00000000;
        const STRENGTH_2 = 0b00001000;
        const STRENGTH_3 = 0b00010000;
        const STRENGTH_4 = 0b00011000;
        const STRENGTH_5 = 0b00100000;
        const STRENGTH_6 = 0b00101000;
        const STRENGTH_7 = 0b00110000;
        const STRENGTH_8 = 0b00111000;

        const OFF_0_27US = 0b00000000;
        const OFF_0_34US = 0b00000001;
        const OFF_0_40US = 0b00000010;
        const OFF_0_54US = 0b00000011;
        const OFF_0_80US = 0b00000100;
        const OFF_1_54US = 0b00000101;
        const OFF_3_34US = 0b00000110;
        const OFF_6_58US = 0b00000111;
    }

    #[repr(transparent)]
    pub struct PfsFlags: u8 {
        const FRAMES_1   = 0b00000000;
        const FRAMES_2   = 0b00010000;
        const FRAMES_3   = 0b00100000;
        const FRAMES_4   = 0b00110000;
    }

    #[repr(transparent)]
    pub struct TseFlags: u8 {
        const TEMP_INTERNAL = 0b00000000;
        const TEMP_EXTERNAL = 0b10000000;

        const OFFSET_0      = 0b00000000;
        const OFFSET_1      = 0b00000001;
        const OFFSET_2      = 0b00000010;
        const OFFSET_3      = 0b00000011;
        const OFFSET_4      = 0b00000100;
        const OFFSET_5      = 0b00000101;
        const OFFSET_6      = 0b00000110;
        const OFFSET_7      = 0b00000111;

        const OFFSET_MIN_8  = 0b00001000;
        const OFFSET_MIN_7  = 0b00001001;
        const OFFSET_MIN_6  = 0b00001010;
        const OFFSET_MIN_5  = 0b00001011;
        const OFFSET_MIN_4  = 0b00001100;
        const OFFSET_MIN_3  = 0b00001101;
        const OFFSET_MIN_2  = 0b00001110;
        const OFFSET_MIN_1  = 0b00001111;
    }

    #[repr(transparent)]
    pub struct PllFlags: u8 {
        // other frequency options exist but there doesn't seem to be much
        // point in including them - this is a fair range of options...
        const HZ_29      = 0b00111111;
        const HZ_33      = 0b00111110;
        const HZ_40      = 0b00111101;
        const HZ_50      = 0b00111100;
        const HZ_67      = 0b00111011;
        const HZ_100     = 0b00111010;
        const HZ_200     = 0b00111001;
    }
}

impl<'a> Uc8151<'a> {
    pub fn new(
        spi: SPI0,
        cs: PIN_17,
        clk: PIN_18,
        miso: PIN_19,
        mosi: PIN_16,
        dc: PIN_20,
        reset: PIN_21,
        busy: PIN_26,
    ) -> Self {
        let mut config = spi::Config::default();
        config.frequency = 12_000_000;
        Self {
            spi: Spi::new_blocking(spi, clk, miso, mosi, config),
            busy: Input::new(busy, Pull::Up),
            chip_select: Output::new(cs, Level::High),
            dc: Output::new(dc, Level::Low),
            reset: Output::new(reset, Level::High),
        }
    }

    pub fn command(&mut self, register: Register, data: &[u8]) {
        self.chip_select.set_low();
        self.dc.set_low();
        self.spi.blocking_write(&[register as u8]);
        self.dc.set_high();
        self.spi.blocking_write(data);
        self.chip_select.set_high();
    }

    pub async fn busy_wait(&mut self) {
        self.busy.wait_for_high().await;
    }

    pub async fn reset(&mut self) {
        self.reset.set_low();
        Timer::after_millis(10).await;
        self.reset.set_high();
        Timer::after_millis(10).await;
        self.busy_wait().await;
    }

    pub async fn init(&mut self) {
        self.reset().await;

        self.command(
            Register::PSR,
            &[(PsrFlags::RES_128x296
                | PsrFlags::LUT_OTP
                | PsrFlags::FORMAT_BW
                | PsrFlags::SHIFT_RIGHT
                | PsrFlags::BOOSTER_ON
                | PsrFlags::RESET_NONE)
                .bits()],
        );

        self.command(
            Register::PWR,
            &[
                (PwrFlags1::VDS_INTERNAL | PwrFlags1::VDG_INTERNAL).bits(),
                (PwrFlags2::VCOM_VD | PwrFlags2::VGHL_16V).bits(),
                0b101011,
                0b101011,
                0b101011,
            ],
        );

        self.command(Register::PON, &[]);

        self.busy_wait().await;

        self.command(
            Register::BTST,
            &[
                (BoosterFlags::START_10MS | BoosterFlags::STRENGTH_3 | BoosterFlags::OFF_6_58US)
                    .bits(),
                (BoosterFlags::START_10MS | BoosterFlags::STRENGTH_3 | BoosterFlags::OFF_6_58US)
                    .bits(),
                (BoosterFlags::START_10MS | BoosterFlags::STRENGTH_3 | BoosterFlags::OFF_6_58US)
                    .bits(),
            ],
        );

        self.command(Register::PFS, &[PfsFlags::FRAMES_1.bits()]);
        self.command(Register::TSE, &[
            (TseFlags::TEMP_INTERNAL | TseFlags::OFFSET_0).bits()
        ]);

        // tcon setting
        self.command(Register::TCON, &[0x22]);

        // vcom and data interval
        self.command(Register::CDI, &[0b01001100]);

        self.command(Register::PLL, &[PllFlags::HZ_100.bits()]);

        // turn off
        self.command(Register::POF, &[]);
        self.busy_wait().await;
    }

    pub async fn update(&mut self, framebuffer: &[u8]) {
        // turn on
		self.command(Register::PON, &[]);

        // disable partial mode
		self.command(Register::PTOU, &[]);

        // transmit framebuffer
		self.command(Register::DTM2, framebuffer);
        // data stop
		self.command(Register::DSP, &[]);

		self.command(Register::DRF, &[]); // start display refresh

        self.busy_wait().await;

		self.command(Register::POF, &[]); // turn off
    }
}
