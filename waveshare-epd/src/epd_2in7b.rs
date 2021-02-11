//! 2.7" 3-color

use crate::Result;
use image::Pixel;
use log::{debug, warn};
use rppal::{
    gpio::{Gpio, InputPin, OutputPin},
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use std::time::Duration;
use std::{iter::repeat, thread};

const RST_PIN: u8 = 17;
const DC_PIN: u8 = 25;
const CS_PIN: u8 = 8;
const BUSY_PIN: u8 = 24;

const EPD_WIDTH: usize = 176;
const EPD_HEIGHT: usize = 264;
const EPD_BUFFER_SIZE: usize = EPD_WIDTH * EPD_HEIGHT / 8;

pub struct Epd {
    reset_pin: OutputPin,
    dc_pin: OutputPin,
    cs_pin: OutputPin,
    busy_pin: InputPin,
    spi: Spi,
}
impl Epd {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new()?;
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;
        Ok(Self {
            reset_pin: gpio.get(RST_PIN)?.into_output(),
            dc_pin: gpio.get(DC_PIN)?.into_output(),
            cs_pin: gpio.get(CS_PIN)?.into_output(),
            busy_pin: gpio.get(BUSY_PIN)?.into_input(),
            spi,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.reset();

        self.read_busy();

        self.send_command(0x4D)?;
        self.send_data(0xAA)?;

        self.send_command(0x87)?;
        self.send_data(0x28)?;

        self.send_command(0x84)?;
        self.send_data(0x00)?;

        self.send_command(0x83)?;
        self.send_data(0x05)?;

        self.send_command(0xA8)?;
        self.send_data(0xDF)?;

        self.send_command(0xA9)?;
        self.send_data(0x05)?;

        self.send_command(0xB1)?;
        self.send_data(0xE8)?;

        self.send_command(0xAB)?;
        self.send_data(0xA1)?;

        self.send_command(0xB9)?;
        self.send_data(0x10)?;

        self.send_command(0x88)?;
        self.send_data(0x80)?;

        self.send_command(0x90)?;
        self.send_data(0x02)?;

        self.send_command(0x86)?;
        self.send_data(0x15)?;

        self.send_command(0x91)?;
        self.send_data(0x8D)?;

        self.send_command(0x50)?;
        self.send_data(0x57)?;

        self.send_command(0xAA)?;
        self.send_data(0x0F)?;

        self.send_command(0x00)?;
        self.send_data(0x8F)?;
        Ok(())
    }

    pub fn display(
        &mut self,
        black: impl Iterator<Item = u8>,
        red: impl Iterator<Item = u8>,
    ) -> Result<()> {
        self.send_command(0x10)?;
        for b in black.take(EPD_BUFFER_SIZE) {
            self.send_data(b)?;
        }

        self.send_command(0x13)?;
        for r in red.take(EPD_BUFFER_SIZE) {
            self.send_data(!r)?;
        }

        self.send_command(0x04)?; // Power ON
        self.read_busy();
        thread::sleep(Duration::from_millis(10));
        self.send_command(0x12)?; // Display Refresh
        self.read_busy();
        thread::sleep(Duration::from_millis(10));
        self.send_command(0x02)?; // Power OFF
        self.read_busy();
        thread::sleep(Duration::from_millis(20));
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.display(repeat(0xFF), repeat(0xFF))
    }

    pub fn sleep(&mut self) -> Result<()> {
        self.send_command(0x07)?;
        self.send_data(0xA5)?;
        Ok(())
    }

    fn reset(&mut self) {
        self.reset_pin.set_high();
        thread::sleep(Duration::from_millis(200));
        self.reset_pin.set_low();
        thread::sleep(Duration::from_millis(5));
        self.reset_pin.set_high();
        thread::sleep(Duration::from_millis(200));
    }

    fn send_command(&mut self, command: u8) -> Result<()> {
        self.dc_pin.set_low();
        self.cs_pin.set_low();
        self.spi.write(&[command])?;
        self.cs_pin.set_high();
        Ok(())
    }

    fn send_data(&mut self, data: u8) -> Result<()> {
        self.dc_pin.set_high();
        self.cs_pin.set_low();
        self.spi.write(&[data])?;
        self.cs_pin.set_high();
        Ok(())
    }

    fn read_busy(&self) {
        debug!("e-Paper busy");
        while self.busy_pin.is_low() {
            thread::sleep(Duration::from_millis(100));
        }
        debug!("e-Paper busy release")
    }
}

impl Drop for Epd {
    fn drop(&mut self) {
        debug!("close 5V, Module enters 0 power consumption ...");
        self.reset_pin.set_low();
        self.dc_pin.set_low();
    }
}

type BwImage = image::GrayImage;
pub fn pack_buffer(image: &BwImage) -> Option<[u8; EPD_BUFFER_SIZE]> {
    if image.width() as usize == EPD_WIDTH && image.height() as usize == EPD_HEIGHT {
        debug!("Vertical");
        let mut buf = [0xFFu8; EPD_BUFFER_SIZE];
        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel.channels()[0] == 0 {
                set_pixel(&mut buf, x as usize, y as usize);
            }
        }
        Some(buf)
    } else if image.width() as usize == EPD_HEIGHT && image.height() as usize == EPD_WIDTH {
        debug!("Horizontal");
        let mut buf = [0xFF; EPD_BUFFER_SIZE];
        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel.channels()[0] == 0 {
                set_pixel(&mut buf, y as usize, EPD_HEIGHT - x as usize - 1);
            }
        }
        Some(buf)
    } else {
        warn!("Unsupported image size {:?}", image.dimensions());
        None
    }
}
fn set_pixel(buf: &mut [u8], x: usize, y: usize) {
    buf[(x + y * EPD_WIDTH) / 8] &= !(0x80 >> (x % 8));
}
