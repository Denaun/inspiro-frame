// ! 12.48" 3-color

use crate::Result;
#[cfg(esp32)]
use esp_idf_hal::{gpio, spi::SPI3};
use esp_idf_hal::{
    gpio::{AnyInputPin, AnyOutputPin, Input, InputPin, Output, OutputPin, PinDriver},
    peripheral::{Peripheral, PeripheralRef},
    spi::{SpiAnyPins, SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
};
use futures::future;
use log::info;
use std::{thread, time::Duration};

pub const EPD_WIDTH: usize = 1304;
pub const EPD_HEIGHT: usize = 984;

pub const LEFT_WIDTH: usize = 648;
pub const RIGHT_WIDTH: usize = EPD_WIDTH - LEFT_WIDTH;
pub const HALF_HEIGHT: usize = EPD_HEIGHT / 2;

const LEFT_BYTES: usize = LEFT_WIDTH / 8;
const RIGHT_BYTES: usize = RIGHT_WIDTH / 8;

pub struct Epd<'d> {
    spi: SpiDriver<'d>,
    m1_cs: PeripheralRef<'d, AnyOutputPin>,
    s1_cs: PeripheralRef<'d, AnyOutputPin>,
    m2_cs: PeripheralRef<'d, AnyOutputPin>,
    s2_cs: PeripheralRef<'d, AnyOutputPin>,
    m1s1_dc: PinDriver<'d, AnyOutputPin, Output>,
    m2s2_dc: PinDriver<'d, AnyOutputPin, Output>,
    m1s1_rst: PinDriver<'d, AnyOutputPin, Output>,
    m2s2_rst: Option<PinDriver<'d, AnyOutputPin, Output>>,
    m1_busy: PinDriver<'d, AnyInputPin, Input>,
    s1_busy: PinDriver<'d, AnyInputPin, Input>,
    m2_busy: PinDriver<'d, AnyInputPin, Input>,
    s2_busy: PinDriver<'d, AnyInputPin, Input>,
}

impl<'d> Epd<'d> {
    #[cfg(esp32)]
    pub fn waveshare(spi: impl Peripheral<P = SPI3> + 'd, pins: gpio::Pins) -> Result<Self> {
        Ok(Self {
            spi: SpiDriver::new(
                spi,
                pins.gpio13,
                pins.gpio14,
                None::<gpio::Gpio12>,
                &SpiDriverConfig::new(),
            )?,
            m1_cs: pins.gpio23.downgrade_output().into_ref(),
            s1_cs: pins.gpio22.downgrade_output().into_ref(),
            m2_cs: pins.gpio16.downgrade_output().into_ref(),
            s2_cs: pins.gpio19.downgrade_output().into_ref(),
            m1s1_dc: PinDriver::output(pins.gpio25.downgrade_output())?,
            m2s2_dc: PinDriver::output(pins.gpio17.downgrade_output())?,
            m1s1_rst: PinDriver::output(pins.gpio33.downgrade_output())?,
            m2s2_rst: Some(PinDriver::output(pins.gpio5.downgrade_output())?),
            m1_busy: PinDriver::input(pins.gpio32.downgrade_input())?,
            s1_busy: PinDriver::input(pins.gpio26.downgrade_input())?,
            m2_busy: PinDriver::input(pins.gpio18.downgrade_input())?,
            s2_busy: PinDriver::input(pins.gpio4.downgrade_input())?,
        })
    }

    pub fn custom(
        spi: impl Peripheral<P = impl SpiAnyPins> + 'd,
        sclk: impl OutputPin + 'd,
        sdo: impl OutputPin + 'd,
        m1_cs: impl OutputPin + 'd,
        s1_cs: impl OutputPin + 'd,
        m2_cs: impl OutputPin + 'd,
        s2_cs: impl OutputPin + 'd,
        m1s1_dc: impl OutputPin + 'd,
        m2s2_dc: impl OutputPin + 'd,
        m1s1_rst: impl OutputPin + 'd,
        m2s2_rst: Option<impl OutputPin + 'd>,
        m1_busy: impl InputPin + 'd,
        s1_busy: impl InputPin + 'd,
        m2_busy: impl InputPin + 'd,
        s2_busy: impl InputPin + 'd,
    ) -> Result<Self> {
        Ok(Self {
            spi: SpiDriver::new(spi, sclk, sdo, None::<AnyInputPin>, &SpiDriverConfig::new())?,
            m1_cs: m1_cs.downgrade_output().into_ref(),
            s1_cs: s1_cs.downgrade_output().into_ref(),
            m2_cs: m2_cs.downgrade_output().into_ref(),
            s2_cs: s2_cs.downgrade_output().into_ref(),
            m1s1_dc: PinDriver::output(m1s1_dc.downgrade_output())?,
            m2s2_dc: PinDriver::output(m2s2_dc.downgrade_output())?,
            m1s1_rst: PinDriver::output(m1s1_rst.downgrade_output())?,
            m2s2_rst: match m2s2_rst {
                Some(pin) => Some(PinDriver::output(pin.downgrade_output())?),
                None => None,
            },
            m1_busy: PinDriver::input(m1_busy.downgrade_input())?,
            s1_busy: PinDriver::input(s1_busy.downgrade_input())?,
            m2_busy: PinDriver::input(m2_busy.downgrade_input())?,
            s2_busy: PinDriver::input(s2_busy.downgrade_input())?,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.reset()?;
        PinDriver::output(self.m1_cs.reborrow())?.set_high()?;
        PinDriver::output(self.s1_cs.reborrow())?.set_high()?;
        PinDriver::output(self.m2_cs.reborrow())?.set_high()?;
        PinDriver::output(self.s2_cs.reborrow())?.set_high()?;
        self.init_v1()
    }
    fn init_v1(&mut self) -> Result<()> {
        info!("Init V1");
        // panel setting
        // KW-3f    KWR-2F   BWROTP 0f   BWOTP 1f
        self.m1s1m2s2_send_command(0x00)?;
        self.m1s1_send_data(&[0x2f])?;
        self.m2s2_send_data(&[0x23])?;

        // POWER SETTING
        // VGH=20V,VGL=-20V
        // VDH=15V
        // VDL=-15V
        self.m1m2_send_command(0x01)?;
        self.m1m2_send_data(&[0x07, 0x17, 0x3F, 0x3F, 0x0d])?;

        // booster soft start
        self.m1m2_send_command(0x06)?;
        self.m1m2_send_data(&[0x17, 0x17, 0x39, 0x17])?;

        // resolution setting
        self.m1s1m2s2_send_command(0x61)?;
        // source 648
        // gate 492
        self.m1s2_send_data(&[0x02, 0x88, 0x01, 0xEC])?;
        // source 656
        // gate 492
        self.s1m2_send_data(&[0x02, 0x90, 0x01, 0xEC])?;

        // DUSPI
        self.m1s1m2s2_send_command(0x15)?;
        self.m1s1m2s2_send_data(&[0x20])?;

        // PLL
        self.m1s1m2s2_send_command(0x30)?;
        self.m1s1m2s2_send_data(&[0x08])?;

        // Vcom and data interval setting
        self.m1s1m2s2_send_command(0x50)?;
        self.m1s1m2s2_send_data(&[0x31, 0x07])?;

        // TCON
        self.m1s1m2s2_send_command(0x60)?;
        self.m1s1m2s2_send_data(&[0x22])?;

        // POWER SETTING
        self.m1m2_send_command(0xE0)?;
        self.m1m2_send_data(&[0x01])?;

        self.m1s1m2s2_send_command(0xE3)?;
        self.m1s1m2s2_send_data(&[0x00])?;

        self.m1m2_send_command(0x82)?;
        self.m1m2_send_data(&[0x1c])?;

        self.set_lut()?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        // M1 part 648*492
        // S1 part 656*492
        // M2 part 656*492
        // S2 part 648*492
        self.m1s1m2s2_send_command(0x10)?;
        self.s2_send_data(&[0xff; LEFT_BYTES * HALF_HEIGHT])?;
        self.m2_send_data(&[0xff; RIGHT_BYTES * HALF_HEIGHT])?;
        self.m1_send_data(&[0xff; LEFT_BYTES * HALF_HEIGHT])?;
        self.s1_send_data(&[0xff; RIGHT_BYTES * HALF_HEIGHT])?;

        self.m1s1m2s2_send_command(0x13)?;
        self.s2_send_data(&[0x00; LEFT_BYTES * HALF_HEIGHT])?;
        self.m2_send_data(&[0x00; RIGHT_BYTES * HALF_HEIGHT])?;
        self.m1_send_data(&[0x00; LEFT_BYTES * HALF_HEIGHT])?;
        self.s1_send_data(&[0x00; RIGHT_BYTES * HALF_HEIGHT])?;

        Ok(())
    }

    /// Write the bottom left white buffer.
    pub fn m1_display_white(&mut self, white: &[u8; LEFT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.m1_send_command(0x10)?;
        self.m1_send_data(white)?;
        Ok(())
    }
    /// Write the bottom left red buffer.
    pub fn m1_display_red(&mut self, red: &[u8; LEFT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.m1_send_command(0x13)?;
        self.m1_send_data(red)?;
        Ok(())
    }

    /// Write the bottom right white buffer.
    pub fn s1_display_white(&mut self, white: &[u8; RIGHT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.s1_send_command(0x10)?;
        self.s1_send_data(white)?;
        Ok(())
    }
    /// Write the bottom right red buffer.
    pub fn s1_display_red(&mut self, red: &[u8; RIGHT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.s1_send_command(0x13)?;
        self.s1_send_data(red)?;
        Ok(())
    }

    /// Write the top right white buffer.
    pub fn m2_display_white(&mut self, white: &[u8; RIGHT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.m2_send_command(0x10)?;
        self.m2_send_data(white)?;
        Ok(())
    }
    /// Write the top right red buffer.
    pub fn m2_display_red(&mut self, red: &[u8; RIGHT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.m2_send_command(0x13)?;
        self.m2_send_data(red)?;
        Ok(())
    }

    /// Write the top left white buffer.
    pub fn s2_display_white(&mut self, white: &[u8; LEFT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.s2_send_command(0x10)?;
        self.s2_send_data(white)?;
        Ok(())
    }
    /// Write the top left red buffer.
    pub fn s2_display_red(&mut self, red: &[u8; LEFT_BYTES * HALF_HEIGHT]) -> Result<()> {
        self.s2_send_command(0x13)?;
        self.s2_send_data(red)?;
        Ok(())
    }

    pub async fn turn_on(&mut self) -> Result<()> {
        self.m1m2_send_command(0x04)?; // power on
        thread::sleep(Duration::from_millis(300));
        self.m1s1m2s2_send_command(0x12)?; // Display Refresh

        info!("Busy");
        self.m1s1m2s2_send_command(0x71)?;
        future::try_join4(
            self.m1_busy.wait_for_high(),
            self.s1_busy.wait_for_high(),
            self.m2_busy.wait_for_high(),
            self.s2_busy.wait_for_high(),
        )
        .await?;
        info!("Busy free");
        Ok(())
    }

    pub fn sleep(&mut self) -> Result<()> {
        // power off
        self.m1s1m2s2_send_command(0x02)?;
        thread::sleep(Duration::from_millis(300));

        // deep sleep
        self.m1s1m2s2_send_command(0x07)?;
        self.m1s1m2s2_send_data(&[0xA5])?;
        thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.m1s1_rst.set_high()?;
        if let Some(pin) = &mut self.m2s2_rst {
            pin.set_high()?;
        }
        thread::sleep(Duration::from_millis(200));
        self.m1s1_rst.set_low()?;
        if let Some(pin) = &mut self.m2s2_rst {
            pin.set_low()?;
        }
        thread::sleep(Duration::from_millis(5));
        self.m1s1_rst.set_high()?;
        if let Some(pin) = &mut self.m2s2_rst {
            pin.set_high()?;
        }
        thread::sleep(Duration::from_millis(200));
        Ok(())
    }

    fn m1_send_command(&mut self, reg: u8) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.m1_cs.reborrow()), &spi_config())?;

        self.m1s1_dc.set_low()?;
        spi.write(&[reg])?;
        Ok(())
    }
    fn m1_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.m1_cs.reborrow()), &spi_config())?;

        self.m1s1_dc.set_high()?;
        spi.write(data)?;
        Ok(())
    }

    fn s1_send_command(&mut self, reg: u8) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.s1_cs.reborrow()), &spi_config())?;

        self.m1s1_dc.set_low()?;
        spi.write(&[reg])?;
        Ok(())
    }
    fn s1_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.s1_cs.reborrow()), &spi_config())?;

        self.m1s1_dc.set_high()?;
        spi.write(data)?;
        Ok(())
    }

    fn m2_send_command(&mut self, reg: u8) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.m2_cs.reborrow()), &spi_config())?;

        self.m2s2_dc.set_low()?;
        spi.write(&[reg])?;
        Ok(())
    }
    fn m2_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.m2_cs.reborrow()), &spi_config())?;

        self.m2s2_dc.set_high()?;
        spi.write(data)?;
        Ok(())
    }

    fn s2_send_command(&mut self, reg: u8) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.s2_cs.reborrow()), &spi_config())?;

        self.m2s2_dc.set_low()?;
        spi.write(&[reg])?;
        Ok(())
    }
    fn s2_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, Some(self.s2_cs.reborrow()), &spi_config())?;

        self.m2s2_dc.set_high()?;
        spi.write(data)?;
        Ok(())
    }

    fn m1s2_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut m1_cs = PinDriver::output(self.m1_cs.reborrow())?;
        let mut s2_cs = PinDriver::output(self.s2_cs.reborrow())?;

        self.m1s1_dc.set_high()?;
        self.m2s2_dc.set_high()?;
        m1_cs.set_low()?;
        s2_cs.set_low()?;
        spi.write(data)?;
        m1_cs.set_high()?;
        s2_cs.set_high()?;
        Ok(())
    }
    fn s1m2_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut s1_cs = PinDriver::output(self.s1_cs.reborrow())?;
        let mut m1_cs = PinDriver::output(self.m1_cs.reborrow())?;

        self.m1s1_dc.set_high()?;
        self.m2s2_dc.set_high()?;
        s1_cs.set_low()?;
        m1_cs.set_low()?;
        spi.write(data)?;
        s1_cs.set_high()?;
        m1_cs.set_high()?;
        Ok(())
    }

    fn m1s1_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut m1_cs = PinDriver::output(self.m1_cs.reborrow())?;
        let mut s1_cs = PinDriver::output(self.s1_cs.reborrow())?;

        self.m1s1_dc.set_high()?;
        m1_cs.set_low()?;
        s1_cs.set_low()?;
        spi.write(data)?;
        m1_cs.set_high()?;
        s1_cs.set_high()?;
        Ok(())
    }
    fn m2s2_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut m2_cs = PinDriver::output(self.m2_cs.reborrow())?;
        let mut s2_cs = PinDriver::output(self.s2_cs.reborrow())?;

        self.m2s2_dc.set_high()?;
        m2_cs.set_low()?;
        s2_cs.set_low()?;
        spi.write(data)?;
        m2_cs.set_high()?;
        s2_cs.set_high()?;
        Ok(())
    }

    fn m1m2_send_command(&mut self, reg: u8) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut m1_cs = PinDriver::output(self.m1_cs.reborrow())?;
        let mut m2_cs = PinDriver::output(self.m2_cs.reborrow())?;

        self.m1s1_dc.set_low()?;
        self.m2s2_dc.set_low()?;
        m1_cs.set_low()?;
        m2_cs.set_low()?;
        spi.write(&[reg])?;
        m1_cs.set_high()?;
        m2_cs.set_high()?;
        Ok(())
    }
    fn m1m2_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut m1_cs = PinDriver::output(self.m1_cs.reborrow())?;
        let mut m2_cs = PinDriver::output(self.m2_cs.reborrow())?;

        self.m1s1_dc.set_high()?;
        self.m2s2_dc.set_high()?;
        m1_cs.set_low()?;
        m2_cs.set_low()?;
        spi.write(data)?;
        m1_cs.set_high()?;
        m2_cs.set_high()?;
        Ok(())
    }

    fn m1s1m2s2_send_command(&mut self, reg: u8) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut m1_cs = PinDriver::output(self.m1_cs.reborrow())?;
        let mut s1_cs = PinDriver::output(self.s1_cs.reborrow())?;
        let mut m2_cs = PinDriver::output(self.m2_cs.reborrow())?;
        let mut s2_cs = PinDriver::output(self.s2_cs.reborrow())?;

        self.m1s1_dc.set_low()?;
        self.m2s2_dc.set_low()?;
        m1_cs.set_low()?;
        s1_cs.set_low()?;
        m2_cs.set_low()?;
        s2_cs.set_low()?;
        spi.write(&[reg])?;
        m1_cs.set_high()?;
        s1_cs.set_high()?;
        m2_cs.set_high()?;
        s2_cs.set_high()?;
        Ok(())
    }
    fn m1s1m2s2_send_data(&mut self, data: &[u8]) -> Result<()> {
        let mut spi = SpiDeviceDriver::new(&self.spi, AnyOutputPin::none(), &spi_config())?;
        let mut m1_cs = PinDriver::output(self.m1_cs.reborrow())?;
        let mut s1_cs = PinDriver::output(self.s1_cs.reborrow())?;
        let mut m2_cs = PinDriver::output(self.m2_cs.reborrow())?;
        let mut s2_cs = PinDriver::output(self.s2_cs.reborrow())?;

        self.m1s1_dc.set_high()?;
        self.m2s2_dc.set_high()?;
        m1_cs.set_low()?;
        s1_cs.set_low()?;
        m2_cs.set_low()?;
        s2_cs.set_low()?;
        spi.write(data)?;
        m1_cs.set_high()?;
        s1_cs.set_high()?;
        m2_cs.set_high()?;
        s2_cs.set_high()?;
        Ok(())
    }

    fn set_lut(&mut self) -> Result<()> {
        self.m1s1m2s2_send_command(0x20)?; // vcom
        self.m1s1m2s2_send_data(&LUT_VCOM1)?;

        self.m1s1m2s2_send_command(0x21)?; // red not use
        self.m1s1m2s2_send_data(&LUT_WW1)?;

        self.m1s1m2s2_send_command(0x22)?; // bw r
        self.m1s1m2s2_send_data(&LUT_BW1)?; // bw=r

        self.m1s1m2s2_send_command(0x23)?; // wb w
        self.m1s1m2s2_send_data(&LUT_WB1)?; // wb=w

        self.m1s1m2s2_send_command(0x24)?; // bb b
        self.m1s1m2s2_send_data(&LUT_BB1)?; // bb=b

        self.m1s1m2s2_send_command(0x25)?; // bb b
        self.m1s1m2s2_send_data(&LUT_WW1)?; // bb=b

        Ok(())
    }
}

const LUT_VCOM1: [u8; 60] = [
    0x00, 0x10, 0x10, 0x01, 0x08, 0x01, 0x00, 0x06, 0x01, 0x06, 0x01, 0x05, 0x00, 0x08, 0x01, 0x08,
    0x01, 0x06, 0x00, 0x06, 0x01, 0x06, 0x01, 0x05, 0x00, 0x05, 0x01, 0x1E, 0x0F, 0x06, 0x00, 0x05,
    0x01, 0x1E, 0x0F, 0x01, 0x00, 0x04, 0x05, 0x08, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const LUT_WW1: [u8; 60] = [
    0x91, 0x10, 0x10, 0x01, 0x08, 0x01, 0x04, 0x06, 0x01, 0x06, 0x01, 0x05, 0x84, 0x08, 0x01, 0x08,
    0x01, 0x06, 0x80, 0x06, 0x01, 0x06, 0x01, 0x05, 0x00, 0x05, 0x01, 0x1E, 0x0F, 0x06, 0x00, 0x05,
    0x01, 0x1E, 0x0F, 0x01, 0x08, 0x04, 0x05, 0x08, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const LUT_BW1: [u8; 60] = [
    0xA8, 0x10, 0x10, 0x01, 0x08, 0x01, 0x84, 0x06, 0x01, 0x06, 0x01, 0x05, 0x84, 0x08, 0x01, 0x08,
    0x01, 0x06, 0x86, 0x06, 0x01, 0x06, 0x01, 0x05, 0x8C, 0x05, 0x01, 0x1E, 0x0F, 0x06, 0x8C, 0x05,
    0x01, 0x1E, 0x0F, 0x01, 0xF0, 0x04, 0x05, 0x08, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const LUT_WB1: [u8; 60] = [
    0x91, 0x10, 0x10, 0x01, 0x08, 0x01, 0x04, 0x06, 0x01, 0x06, 0x01, 0x05, 0x84, 0x08, 0x01, 0x08,
    0x01, 0x06, 0x80, 0x06, 0x01, 0x06, 0x01, 0x05, 0x00, 0x05, 0x01, 0x1E, 0x0F, 0x06, 0x00, 0x05,
    0x01, 0x1E, 0x0F, 0x01, 0x08, 0x04, 0x05, 0x08, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const LUT_BB1: [u8; 60] = [
    0x92, 0x10, 0x10, 0x01, 0x08, 0x01, 0x80, 0x06, 0x01, 0x06, 0x01, 0x05, 0x84, 0x08, 0x01, 0x08,
    0x01, 0x06, 0x04, 0x06, 0x01, 0x06, 0x01, 0x05, 0x00, 0x05, 0x01, 0x1E, 0x0F, 0x06, 0x00, 0x05,
    0x01, 0x1E, 0x0F, 0x01, 0x01, 0x04, 0x05, 0x08, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

fn spi_config() -> SpiConfig {
    SpiConfig::new()
}
