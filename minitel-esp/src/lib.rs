use std::io::{Error, ErrorKind, Result};

use esp_idf_hal::{
    gpio::AnyIOPin,
    io::Write,
    sys::{EspError, TickType_t},
    uart,
    units::Hertz,
};
use minitel_stum::{BaudrateControl, Minitel, SerialPort};

pub type ESPMinitel<'a> = Minitel<ESPPort<'a>>;

/// Create a new Minitel instance using the ESP32 UART.
pub fn esp_minitel(uart: uart::UartDriver, read_timeout: TickType_t) -> ESPMinitel {
    ESPMinitel::new(ESPPort::new(uart, read_timeout))
}

/// Serial port configuration when the minitel starts
pub fn default_uart_config() -> uart::UartConfig {
    uart::UartConfig::default()
        .baudrate(Hertz(1200))
        .stop_bits(uart::config::StopBits::STOP1)
        .data_bits(uart::config::DataBits::DataBits7)
        .parity_even()
}

/// Create a new Minitel instance using the port 1 UART.
/// This is the port used in the ESP32 minitel development board from iodeo.
pub fn esp_minitel_uart2() -> core::result::Result<ESPMinitel<'static>, EspError> {
    let peripherals = esp_idf_hal::peripherals::Peripherals::take()?;
    let pins = peripherals.pins;

    let uart: uart::UartDriver = uart::UartDriver::new(
        peripherals.uart2,
        pins.gpio17,
        pins.gpio16,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &default_uart_config(),
    )?;

    Ok(esp_minitel(uart, 5))
}

pub struct ESPPort<'a> {
    pub uart: uart::UartDriver<'a>,
    pub read_timeout: TickType_t,
}

impl<'a> ESPPort<'a> {
    pub fn new(uart: uart::UartDriver<'a>, read_timeout: TickType_t) -> Self {
        Self { uart, read_timeout }
    }
}

impl<'a> SerialPort for ESPPort<'a> {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.uart
            .write_all(data)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    fn read(&mut self, data: &mut [u8]) -> Result<()> {
        self.uart
            .read(data, self.read_timeout)
            .map_err(|e| Error::new(ErrorKind::Other, e))?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.uart
            .flush()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

impl<'a> BaudrateControl for ESPPort<'a> {
    fn set_baudrate(&mut self, baudrate: minitel_stum::protocol::Baudrate) -> Result<()> {
        self.uart
            .change_baudrate(baudrate.hertz())
            .map_err(|e| Error::new(ErrorKind::Other, e))?;
        Ok(())
    }
}
