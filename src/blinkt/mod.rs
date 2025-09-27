// Used by rustdoc to link other crates to blinkt's docs
#![allow(
    clippy::trivially_copy_pass_by_ref,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::error;
use std::fmt;
use std::io;
use std::result;
use std::slice;
use std::time::Duration;

use rppal::gpio::{Gpio, OutputPin};

pub use rppal::gpio::Error as GpioError;
pub use rppal::spi::Error as SpiError;

mod pixel;

pub use pixel::Pixel;

// Default values for the Pimoroni Blinkt! board using BCM GPIO pin numbers
const DAT: u8 = 23;
const CLK: u8 = 24;
const NUM_PIXELS: usize = 8;

#[derive(Debug)]
/// Errors that can occur while using Blinkt.
pub enum Error {
    /// Accessing the GPIO peripheral returned an error.
    ///
    /// Some of these errors can be fixed by changing file permissions, or upgrading
    /// to a more recent version of Raspbian.
    Gpio(GpioError),
    /// Accessing the SPI peripheral returned an error.
    Spi(SpiError),
    /// An I/O operation returned an error.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Gpio(ref err) => write!(f, "GPIO error: {err}"),
            Self::Spi(ref err) => write!(f, "SPI error: {err}"),
            Self::Io(ref err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<GpioError> for Error {
    fn from(err: GpioError) -> Self {
        Self::Gpio(err)
    }
}

impl From<SpiError> for Error {
    fn from(err: SpiError) -> Self {
        Self::Spi(err)
    }
}

/// Result type returned from methods that can have `blinkt::Error`s.
pub type Result<T> = result::Result<T, Error>;

trait SerialOutput {
    fn write(&mut self, data: &[u8]) -> Result<()>;
}

struct BlinktGpio {
    pin_data: OutputPin,
    pin_clock: OutputPin,
}

impl BlinktGpio {
    pub fn with_settings(pin_data: u8, pin_clock: u8) -> Result<Self> {
        let gpio = Gpio::new()?;

        let mut pin_data = gpio.get(pin_data)?.into_output();
        let mut pin_clock = gpio.get(pin_clock)?.into_output();

        pin_data.set_low();
        pin_clock.set_low();

        Ok(Self {
            pin_data,
            pin_clock,
        })
    }
}

impl SerialOutput for BlinktGpio {
    fn write(&mut self, data: &[u8]) -> Result<()> {
        for byte in data {
            for n in 0..8 {
                if (byte & (1 << (7 - n))) > 0 {
                    self.pin_data.set_high();
                } else {
                    self.pin_data.set_low();
                }

                self.pin_clock.set_high();
                std::thread::sleep(Duration::from_nanos(10000));
                self.pin_clock.set_low();
            }
        }

        Ok(())
    }
}

pub mod spi {
    pub use rppal::spi::Spi;
    pub use rppal::spi::{Bus, Mode, SlaveSelect};
}

pub struct BlinktSpi(spi::Spi);

impl BlinktSpi {
    pub fn with_settings(
        bus: spi::Bus,
        slave: spi::SlaveSelect,
        clock_speed_hz: u32,
        mode: spi::Mode,
    ) -> Result<Self> {
        Ok(Self(spi::Spi::new(bus, slave, clock_speed_hz, mode)?))
    }
}

impl Default for BlinktSpi {
    fn default() -> Self {
        Self(
            spi::Spi::new(
                spi::Bus::Spi0,
                spi::SlaveSelect::Ss0,
                1_000_000,
                spi::Mode::Mode0,
            )
            .expect("Can't create spi bus"),
        )
    }
}

impl SerialOutput for BlinktSpi {
    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.0.write(data)?;

        Ok(())
    }
}

/// Interface for the Pimoroni Blinkt!, and any similar APA102 or SK9822 LED
/// strips or boards.
///
/// By default, Blinkt is set up to communicate with an 8-pixel board through
/// data pin GPIO 23 (physical pin 16) and clock pin GPIO 24 (physical pin 18).
/// These settings can be changed to support alternate configurations.
pub struct Blinkt {
    serial_output: Box<dyn SerialOutput + Send>,
    pixels: Vec<Pixel>,
    clear_on_drop: bool,
    end_frame: Vec<u8>,
}

impl Blinkt {
    /// Constructs a new `Blinkt` using the default settings for a Pimoroni
    /// Blinkt! board.
    ///
    /// This sets the data pin to GPIO 23 (physical pin 16), the clock pin to
    /// GPIO 24 (physical pin 18), and number of pixels to 8.
    pub fn new() -> Result<Self> {
        Self::with_settings(DAT, CLK, NUM_PIXELS)
    }

    /// Constructs a new `Blinkt` using bitbanging mode, with custom settings for
    /// the data pin, clock pin, and number of pixels. Pins should be specified
    /// by their BCM GPIO pin numbers.
    pub fn with_settings(pin_data: u8, pin_clock: u8, num_pixels: usize) -> Result<Self> {
        Ok(Self {
            serial_output: Box::new(BlinktGpio::with_settings(pin_data, pin_clock)?),
            pixels: vec![Pixel::default(); num_pixels],
            clear_on_drop: true,
            end_frame: vec![0u8; 4 + (((num_pixels as f32 / 16.0f32) + 0.94f32) as usize)],
        })
    }

    /// Constructs a new `Blinkt` using hardware SPI, with custom settings for the
    /// clock speed and number of pixels.
    ///
    /// This sets the data pin to GPIO 10 (physical pin 19) and the clock pin
    /// to GPIO 11 (physical pin 23).
    ///
    /// The Raspberry Pi allows SPI clock speeds up to 125 MHz (125_000_000),
    /// but the maximum speed supported by LED strips depends a lot on the
    /// number of pixels and wire quality, and requires some experimentation.
    /// 32 MHz (32_000_000) seems to be the maximum clock speed for a typical
    /// short LED strip. Visit the [Raspberry Pi SPI Documentation](https://www.raspberrypi.org/documentation/hardware/raspberrypi/spi/)
    /// page for a complete list of supported clock speeds.
    pub fn with_spi(spi: BlinktSpi, num_pixels: usize) -> Self {
        Self {
            serial_output: Box::new(spi),
            pixels: vec![Pixel::default(); num_pixels],
            clear_on_drop: true,
            end_frame: vec![0u8; 4 + (((num_pixels as f32 / 16.0f32) + 0.94f32) as usize)],
        }
    }

    /// Returns a mutable iterator over all `Pixel`s stored in `Blinkt`.
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Pixel> {
        self.pixels.iter_mut()
    }

    /// Sets the red, green and blue values for a single pixel in the local
    /// buffer.
    ///
    /// Pixels are numbered starting at `0`.
    /// `red`, `green` and `blue` are specified as 8-bit values between `0` (0%) and `255` (100%).
    pub fn set_pixel(&mut self, pixel: usize, red: u8, green: u8, blue: u8) {
        if let Some(pixel) = self.pixels.get_mut(pixel) {
            pixel.set_rgb(red, green, blue);
        }
    }

    /// Sets the red, green, blue and brightness values for a single pixel in
    /// the local buffer.
    ///
    /// Pixels are numbered starting at `0`.
    /// `red`, `green` and `blue` are specified as 8-bit values between `0` (0%) and `255` (100%).
    /// `brightness` is specified as a floating point value between `0.0` (0%) and `1.0` (100%), and is converted to a 5-bit value.
    pub fn set_pixel_rgbb(&mut self, pixel: usize, red: u8, green: u8, blue: u8, brightness: f32) {
        if let Some(pixel) = self.pixels.get_mut(pixel) {
            pixel.set_rgbb(red, green, blue, brightness);
        }
    }

    /// Sets the brightness value for a single pixel in the local buffer.
    ///
    /// Pixels are numbered starting at `0`.
    /// `brightness` is specified as a floating point value between `0.0` (0%) and `1.0` (100%), and is converted to a 5-bit value.
    pub fn set_pixel_brightness(&mut self, pixel: usize, brightness: f32) {
        if let Some(pixel) = self.pixels.get_mut(pixel) {
            pixel.set_brightness(brightness);
        }
    }

    /// Sets the red, green and blue values for all pixels in the local buffer.
    ///
    /// `red`, `green` and `blue` are specified as 8-bit values between `0` (0%) and `255` (100%).
    pub fn set_all_pixels(&mut self, red: u8, green: u8, blue: u8) {
        for pixel in &mut self.pixels {
            pixel.set_rgb(red, green, blue);
        }
    }

    /// Sets the red, green, blue and brightness values for all pixels in the
    /// local buffer.
    ///
    /// `red`, `green` and `blue` are specified as 8-bit values between `0` (0%) and `255` (100%).
    /// `brightness` is specified as a floating point value between `0.0` (0%) and `1.0` (100%), and is converted to a 5-bit value.
    pub fn set_all_pixels_rgbb(&mut self, red: u8, green: u8, blue: u8, brightness: f32) {
        for pixel in &mut self.pixels {
            pixel.set_rgbb(red, green, blue, brightness);
        }
    }

    /// Sets the brightness value for all pixels.
    ///
    /// `brightness` is specified as a floating point value between `0.0` (0%) and `1.0` (100%), and is converted to a 5-bit value.
    pub fn set_all_pixels_brightness(&mut self, brightness: f32) {
        for pixel in &mut self.pixels {
            pixel.set_brightness(brightness);
        }
    }

    /// Sets the red, green and blue values for all pixels to `0`.
    pub fn clear(&mut self) {
        self.set_all_pixels(0, 0, 0);
    }

    /// Sends the contents of the local buffer to the pixels, updating their
    /// LED colors and brightness.
    pub fn show(&mut self) -> Result<()> {
        // Start frame (32*0).
        self.serial_output.write(&[0u8; 4])?;

        // LED frames (3*1, 5*brightness, 8*blue, 8*green, 8*red).
        for pixel in &self.pixels {
            self.serial_output.write(pixel.bytes())?;
        }

        // End frame (8*0 for every 16 pixels, 32*0 SK9822 reset frame).
        // The SK9822 won't update any pixels until it receives the next
        // start frame (32*0). The APA102 doesn't care if we send zeroes
        // instead of ones as the end frame. This workaround is
        // compatible with both the APA102 and SK9822.
        self.serial_output.write(&self.end_frame)?;

        Ok(())
    }

    /// Returns the value of `clear_on_drop`.
    pub const fn clear_on_drop(&self) -> bool {
        self.clear_on_drop
    }

    /// When enabled, clears all pixels when `Blinkt` goes out of scope.
    ///
    /// By default, this is set to `true`.
    ///
    /// ## Note
    ///
    /// Drop methods aren't called when a process is abnormally terminated, for
    /// instance when a user presses <kbd>Ctrl</kbd> + <kbd>C</kbd>, and the `SIGINT` signal
    /// isn't caught. You can catch those using crates such as [`simple_signal`].
    ///
    /// [`simple_signal`]: https://crates.io/crates/simple-signal
    pub const fn set_clear_on_drop(&mut self, clear_on_drop: bool) {
        self.clear_on_drop = clear_on_drop;
    }
}

impl Drop for Blinkt {
    /// Clears all pixels if [`clear_on_drop`] is set to `true` (default).
    ///
    /// [`clear_on_drop`]: #method.clear_on_drop
    fn drop(&mut self) {
        if self.clear_on_drop {
            self.clear();
            let _ = self.show();
        }
    }
}

impl<'a> IntoIterator for &'a mut Blinkt {
    type Item = &'a mut Pixel;
    type IntoIter = slice::IterMut<'a, Pixel>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
