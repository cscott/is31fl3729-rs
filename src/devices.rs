// #[cfg_attr(docsrs, doc(cfg(feature = "sevensegment")))]
#[allow(unused_imports)]
use crate::{Error, IS31FL3729};
#[allow(unused_imports)]
use core::convert::TryFrom;
#[allow(unused_imports)]
use embedded_hal::blocking::delay::DelayMs;
#[allow(unused_imports)]
use embedded_hal::blocking::i2c::Write;
#[allow(unused_imports)]
use embedded_hal::blocking::i2c::WriteRead;

#[cfg(feature = "sevensegment")]
pub struct SevenSegment<I2C> {
    pub device: IS31FL3729<I2C>,
}

#[cfg(feature = "sevensegment")]
impl<I2C, I2cError> SevenSegment<I2C>
where
    I2C: Write<Error = I2cError>,
    I2C: WriteRead<Error = I2cError>,
{
    pub fn unwrap(self) -> I2C {
        self.device.i2c
    }

    pub fn set_scaling(&mut self, which: u8, scale: u8) -> Result<(), I2cError> {
        self.device.set_scaling(which, scale)
    }

    pub fn configure(i2c: I2C) -> SevenSegment<I2C> {
        SevenSegment {
            device: IS31FL3729 {
                i2c,
                address: 0x34,
                // logically there are 9 "columns" of 7-segment displays
                width: 9,
                height: 8, // 7 segments plus a decimal point
                calc_pixel: |x: u8, y: u8| -> u8 {
                    x + (0x10 * y)
                },
            },
        }
    }

    pub fn set_percent(&mut self, which: u8, val: f32) -> Result<(), Error<I2cError>> {
        let base = which * 3;
        if val >= 100_f32 {
            self.set_digit(base, 1, false)?;
            self.set_digit(base+1, 0, false)?;
            self.set_digit(base+2, 0, false)?;
        } else if val < 10_f32 {
            // Blank the first digit
            for seg in 0..8 {
                self.device.pixel(base, seg, 0x00)?;
            }
            if val > 0_f32 {
                self.set_digit(base+1, ((val) as u8) % 10, true)?;
                self.set_digit(base+2, ((10_f32*val) as u8) % 10, false)?;
            } else {
                self.set_digit(base+1, 0, true)?;
                self.set_digit(base+2, 0, false)?;
            }
        } else {
            self.set_digit(base, ((val/10_f32) as u8) % 10, false)?;
            self.set_digit(base, ((val) as u8) % 10, true)?;
            self.set_digit(base, ((10_f32*val) as u8) % 10, false)?;
        }
        Ok(())
    }

    pub fn set_digit(&mut self, which: u8, val: u8, point: bool) -> Result<(), Error<I2cError>> {
        let lookup: [[bool; 7]; 36] = [
            // 0
            [true, true, true, true, true, true, false ],
            // 1
            [false, true, true, false, false, false, false ],
            // 2
            [true, true, false, true, true, false, true ],
            // 3
            [true, true, true, true, false, false, true ],
            // 4
            [false, true, true, false, false, true, true ],
            // 5
            [true, false, true, true, false, true, true ],
            // 6
            [true, false, true, true, true, true, true ],
            // 7
            [true, true, true, false, false, false, false ],
            // 8
            [true, true, true, true, true, true, true ],
            // 9
            [true, true, true, false, false, true, true ],
            // A
            [true, true, true, false, true, true, true ],
            // b
            [false, false, true, true, true, true, true ],
            // C
            [true, false, false, true, true, true, false ],
            // d
            [false, true, true, true, true, false, true ],
            // E
            [true, false, false, true, true, true, true ],
            // F
            [true, false, false, false, true, true, true ],
            // G
            [true, false, true, true, true, true, false ],
            // H
            [false, true, true, false, true, true, true ],
            // i
            [false, false, true, false, false, false, false ],
            // J
            [false, true, true, true, true, false, false ],
            // K -- n/a
            [false, false, false, false, false, false, true ],
            // L
            [false, false, false, true, true, true, false ],
            // M -- n/a
            [false, false, false, false, false, false, true ],
            // n
            [false, false, true, false, true, false, true ],
            // o
            [false, false, true, true, true, false, true ],
            // P
            [true, true, false, false, true, true, true ],
            // q -- n/a
            [false, false, false, false, false, false, true ],
            // r
            [false, false, false, false, true, false, true ],
            // S (same as 5)
            [true, false, true, true, false, true, true ],
            // t
            [false, false, false, true, true, true, true ],
            // u
            [false, false, true, true, true, false, false ],
            // v -- n/a
            [false, false, false, false, false, false, true ],
            // w -- n/a
            [false, false, false, false, false, false, true ],
            // x -- n/a
            [false, false, false, false, false, false, true ],
            // Y
            [false, true, true, true, false, true, true ],
            // Z (same as 2)
            [true, true, false, true, true, false, true ],
        ];
        for seg in 0..7 {
            self.device.pixel(which, seg, if lookup[(val%36) as usize][seg as usize] { 0xFF } else { 0x00 })?;
        }
        self.device.pixel(which, 7, if point { 0xFF } else { 0x00 })?;
        Ok(())
    }

    pub fn setup<DEL: DelayMs<u8>>(&mut self, delay: &mut DEL) -> Result<(), Error<I2cError>> {
        self.device.setup(delay)
    }
}
