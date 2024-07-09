#![no_std]
#![doc = include_str!("../README.md")]
/// Preconfigured devices
pub mod devices;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;

/// A struct to integrate with a new IS31FL3729 powered device.
pub struct IS31FL3729<I2C> {
    /// The i2c bus that is used to interact with the device. See implementation below for the
    /// trait methods required.
    pub i2c: I2C,
    /// The 7-bit i2c slave address of the device. By default on most devices this is `0x34`.
    pub address: u8,
    /// Width of the LED matrix
    pub width: u8,
    /// Height of the LED matrix
    pub height: u8,
    /// Method to convert an x,y coordinate pair to a binary address that can be accessed using the
    /// bus.
    pub calc_pixel: fn(x: u8, y: u8) -> u8,
}

impl<I2C, I2cError> IS31FL3729<I2C>
where
    I2C: Write<Error = I2cError>,
    I2C: WriteRead<Error = I2cError>,
{
    /// Fill all pixels of the display at once. The brightness should range from 0 to 255.
    pub fn fill_matrix(&mut self, brightnesses: &[u8]) -> Result<(), I2cError> {
        // Extend by one, to add address to the beginning
        let mut buf = [0x00; 1+addresses::PWM_LEN];
        buf[0] = addresses::PWM_BASE_REGISTER; // set the initial address

        buf[1..=addresses::PWM_LEN].copy_from_slice(&brightnesses[..=(addresses::PWM_LEN-1)]);
        self.write(&buf)?;

        Ok(())
    }

    /// Fill the display with a single brightness. The brightness should range from 0 to 255.
    pub fn fill(&mut self, brightness: u8) -> Result<(), I2cError> {
        let mut buf = [brightness; addresses::PWM_LEN+1];
        buf[0] = addresses::PWM_BASE_REGISTER; // set the initial address
        self.write(&buf)?;
        Ok(())
    }

    /// Setup the display. Should be called before interacting with the device to ensure proper
    /// functionality. Delay is something that your device's HAL should provide which allows for
    /// the process to sleep for a certain amount of time (in this case 10 MS to perform a reset).
    ///
    /// When you run this function the following steps will occur:
    /// 1. The chip will be told that it's being "reset".
    /// 2. The chip will be put in shutdown mode
    /// 3. The chip will be configured to use the maximum voltage
    /// 4. The chip will be taken out of shutdown mode
    pub fn setup<DEL: DelayMs<u8>>(&mut self, delay: &mut DEL) -> Result<(), Error<I2cError>> {
        self.reset(delay)?;
        self.shutdown(true)?;
        delay.delay_ms(10);
        // maximum current limiting
        self.write_u8(addresses::GCC_REGISTER, 0x40)?;
        // scaling registers at max
        let mut buf = [0xff; addresses::SCALING_LEN + 1];
        buf[0] = addresses::PWM_BASE_REGISTER;
        self.write(&buf)?;
        self.shutdown(false)?;
        Ok(())
    }

    /// Set the brightness at a specific x,y coordinate. Just like the [fill method](Self::fill)
    /// the brightness should range from 0 to 255. If the coordinate is out of range then the
    /// function will return an error of [InvalidLocation](Error::InvalidLocation).
    pub fn pixel(&mut self, x: u8, y: u8, brightness: u8) -> Result<(), Error<I2cError>> {
        if x > self.width {
            return Err(Error::InvalidLocation(x));
        }
        if y > self.height {
            return Err(Error::InvalidLocation(y));
        }
        let pixel = (self.calc_pixel)(x, y);
        if (pixel as usize) >= addresses::PWM_LEN {
            return Err(Error::InvalidLocation(pixel));
        }
        self.write_u8(addresses::PWM_BASE_REGISTER + pixel, brightness)?;
        Ok(())
    }

    /// Change the slave address to a new 7-bit address. Should be configured before calling
    /// [setup](Self::setup) method.
    pub fn set_address(&mut self, address: u8) {
        self.address = address;
    }

    /// Send a reset message to the slave device. Delay is something that your device's HAL should
    /// provide which allows for the process to sleep for a certain amount of time (in this case 10
    /// MS to perform a reset).
    pub fn reset<DEL: DelayMs<u8>>(&mut self, delay: &mut DEL) -> Result<(), I2cError> {
        self.write_u8(addresses::RESET_REGISTER, addresses::RESET)?;
        delay.delay_ms(10);
        Ok(())
    }

    /// Set the current available to each CSy of LEDs.
    /// cs should be between 0 and 15 inclusive
    /// scale: 0 is none, 255 is the maximum available
    pub fn set_scaling(&mut self, cs: u8, scale: u8) -> Result<(), I2cError> {
        self.write_u8(addresses::SCALING_BASE_REGISTER + cs, scale)?;
        Ok(())
    }

    /// Put the device into software shutdown mode
    pub fn shutdown(&mut self, yes: bool) -> Result<(), I2cError> {
        let config_val = self.read_u8(addresses::CONFIG_REGISTER)?;
        self.write_u8(
            addresses::CONFIG_REGISTER,
            (config_val & 0xFE) |
            (if yes { 0 } else { 1 }),
        )?;
        Ok(())
    }

    /// How many SW rows to enable
    pub fn sw_enablement(&mut self, setting: SwSetting) -> Result<(), I2cError> {
        let config_register = self.read_u8(addresses::CONFIG_REGISTER)?;

        let new_val = (config_register & 0x0F) | (setting as u8) << 4;
        self.write_u8(addresses::CONFIG_REGISTER, new_val)?;
        Ok(())
    }

    /// Set the PWM frequency
    pub fn set_pwm_freq(&mut self, pwm: PwmFreq) -> Result<(), I2cError> {
        self.write_u8(addresses::PWM_FREQ_REGISTER, pwm as u8)
    }

    /// Set the spread spectrum properties
    pub fn set_spread_spectrum(&mut self, enable: bool, range: SspRange, cycle: SspCycleTime) -> Result<(), I2cError> {
        self.write_u8(addresses::SPREAD_SPECTRUM_REGISTER,
                      (if enable { 0x10 } else { 0x00 }) |
                      ((range as u8) << 2) |
                      (cycle as u8))
    }

    /// Check for opens
    pub fn check_opens(&mut self) -> Result<[u8; 18], I2cError> {
        self.check_open_short(true)
    }
    /// Check for shorts
    pub fn check_shorts(&mut self) -> Result<[u8; 18], I2cError> {
        self.check_open_short(false)
    }
    fn check_open_short(&mut self, open: bool) -> Result<[u8; 18], I2cError> {
        let mut buf = [0x00 ; addresses::OPEN_SHORT_LEN];
        let old_config = self.read_u8(addresses::CONFIG_REGISTER)?;
        let old_gcc = self.read_u8(addresses::GCC_REGISTER)?;
        let osde = if open { OSDE::EnableOpen } else { OSDE::EnableShort };
        self.write_u8(addresses::GCC_REGISTER, 0x01)?;
        self.write_u8(addresses::CONFIG_REGISTER,
                      (old_config & 0xF9) | ((osde as u8) << 1))?;
        self.i2c.write_read(
            self.address,
            &[addresses::OPEN_SHORT_BASE_REGISTER],
            &mut buf)?;
        self.write_u8(addresses::CONFIG_REGISTER, old_config & 0xF9)?;
        self.write_u8(addresses::GCC_REGISTER, old_gcc)?;
        Ok(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), I2cError> {
        self.i2c.write(self.address, buf)
    }

    fn write_u8(&mut self, register: u8, value: u8) -> Result<(), I2cError> {
        self.i2c.write(self.address, &[register, value])
    }

    fn read_u8(&mut self, register: u8) -> Result<u8, I2cError> {
        let mut buf = [0x00];
        self.i2c.write_read(self.address, &[register], &mut buf)?;
        Ok(buf[0])
    }
}

/// See the [data sheet](https://lumissil.com/assets/pdf/core/IS31FL3729_DS.pdf)
/// for more information on registers.
pub mod addresses {

    pub const PWM_BASE_REGISTER: u8 = 0x01;
    pub const PWM_LEN: usize = 0x8F;
    pub const SCALING_BASE_REGISTER: u8 = 0x90;
    pub const SCALING_LEN: usize = 0x10;
    pub const CONFIG_REGISTER: u8 = 0xA0;
    // "Global Current Control" register
    pub const GCC_REGISTER: u8 = 0xA1;
    pub const PULL_DOWN_UP_REGISTER: u8 = 0xB0;
    pub const SPREAD_SPECTRUM_REGISTER: u8 = 0xB1;
    pub const PWM_FREQ_REGISTER: u8 = 0xB2;
    pub const OPEN_SHORT_BASE_REGISTER: u8 = 0xB3;
    pub const OPEN_SHORT_LEN: usize = 0x12;
    pub const RESET_REGISTER: u8 = 0xCF;

    pub const SHUTDOWN: u8 = 0x0A;

    pub const CONFIG_WRITE_ENABLE: u8 = 0b1100_0101;
    pub const RESET: u8 = 0xAE;
}

#[derive(Clone, Copy, Debug)]
pub enum Error<I2cError> {
    I2cError(I2cError),
    InvalidLocation(u8),
    InvalidFrame(u8),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::I2cError(error)
    }
}

#[repr(u8)]
pub enum PwmFreq {
    /// 55kHz
    P55k = 0b000,
    /// 32kHz
    P32k = 0b001,
    /// 4kHz
    P4k = 0b010,
    /// 2kHz
    P2k = 0b011,
    /// 1kHz
    P1k = 0b100,
    /// 500Hz
    P500 = 0b101,
    /// 250Hz
    P250 = 0b110,
    /// 80kHz
    P80k = 0b111,
}

#[repr(u8)]
pub enum SwSetting {
    // SW1-SW9 active, 9SWx15CS matrix
    Sw1Sw9 = 0b0000,
    // SW1-SW8 active, 8SWx16CS matrix
    Sw1Sw8 = 0b0001,
    // SW1-SW7 active, 7SWx16CS matrix, SW8 not active
    Sw1Sw7 = 0b0010,
    // SW1-SW6 active, 6SWx16CS matrix, SW7-SW8 not active
    Sw1Sw6 = 0b0011,
    // SW1-SW5 active, 5SWx16CS matrix, SW6-SW8 not active
    Sw1Sw5 = 0b0100,
    // SW1-SW4 active, 4SWx16CS matrix, SW5-SW8 not active
    Sw1Sw4 = 0b0101,
    // SW1-SW3 active, 3SWx16CS matrix, SW4-SW8 not active
    Sw1Sw3 = 0b0110,
    // SW1-SW2 active, 2SWx16CS matrix, SW3-SW8 not active
    Sw1Sw2 = 0b0111,
    // All CSx pins only act as current sink, no scanning
    NoScan = 0b1000,
}

#[repr(u8)]
pub enum OSDE {
    DisableOSD = 0b00,
    EnableOpen = 0b01,
    EnableShort = 0b10,
}

#[repr(u8)]
pub enum SspRange {
    /// +/- 5%
    Range5 = 0b00,
    /// +/- 15%
    Range15 = 0b01,
    /// +/- 24%
    Range24 = 0b10,
    /// +/- 34%
    Range34 = 0b11,
}

#[repr(u8)]
pub enum SspCycleTime {
    /// 1980us
    Cycle1980 = 0b00,
    /// 1200us
    Cycle1200 = 0b01,
    /// 820us
    Cycle820 = 0b10,
    /// 660us
    Cycle660 = 0b11,
}
