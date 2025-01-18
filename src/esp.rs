#[doc(inline)]
pub use esp::*;

#[cfg(feature = "esp")]
mod esp {
    use crate::{AsyncMinitelBaudrateControl, AsyncMinitelRead, AsyncMinitelWrite};
    use esp_idf_hal::{
        gpio::AnyIOPin,
        io::asynch::{Read, Write},
        sys::EspError,
        uart,
        units::Hertz,
    };
    use std::{
        borrow::BorrowMut,
        io::{Error, ErrorKind, Result},
    };

    /// Serial port configuration when the minitel starts
    pub fn default_uart_config() -> uart::UartConfig {
        uart::UartConfig::default()
            .baudrate(Hertz(1200))
            .stop_bits(uart::config::StopBits::STOP1)
            .data_bits(uart::config::DataBits::DataBits7)
            .parity_even()
    }

    /// Create a new Minitel instance using the port UART 2.
    ///
    /// This is the port used in the ESP32 minitel development board from iodeo.
    pub fn esp_minitel_uart2(
    ) -> core::result::Result<Port<'static, uart::UartDriver<'static>>, EspError> {
        let peripherals = esp_idf_hal::peripherals::Peripherals::take()?;
        let pins = peripherals.pins;

        let uart: uart::AsyncUartDriver<'static, uart::UartDriver<'static>> =
            uart::AsyncUartDriver::new(
                peripherals.uart2,
                pins.gpio17,
                pins.gpio16,
                Option::<AnyIOPin>::None,
                Option::<AnyIOPin>::None,
                &default_uart_config(),
            )?;

        Ok(Port::new(uart))
    }

    pub struct Port<'a, T>
    where
        T: BorrowMut<uart::UartDriver<'a>>,
    {
        pub uart: uart::AsyncUartDriver<'a, T>,
    }

    impl<'a, T> Port<'a, T>
    where
        T: BorrowMut<uart::UartDriver<'a>>,
    {
        pub fn new(uart: uart::AsyncUartDriver<'a, T>) -> Self {
            Port { uart }
        }
    }

    impl<'a, T> AsyncMinitelRead for Port<'a, T>
    where
        T: BorrowMut<uart::UartDriver<'a>>,
    {
        async fn read(&mut self, data: &mut [u8]) -> Result<()> {
            self.uart
                .read_exact(data)
                .await
                .map_err(|e| Error::new(ErrorKind::Other, e))
        }
    }

    impl<'a, T> AsyncMinitelWrite for Port<'a, T>
    where
        T: BorrowMut<uart::UartDriver<'a>>,
    {
        async fn write(&mut self, data: &[u8]) -> Result<()> {
            self.uart
                .write_all(data)
                .await
                .map_err(|e| Error::new(ErrorKind::Other, e))
        }

        async fn flush(&mut self) -> Result<()> {
            self.uart
                .flush()
                .await
                .map_err(|e| Error::new(ErrorKind::Other, e))
        }
    }

    impl<'a, T> AsyncMinitelBaudrateControl for Port<'a, T>
    where
        T: BorrowMut<uart::UartDriver<'a>>,
    {
        fn set_baudrate(&mut self, baudrate: crate::stum::protocol::Baudrate) -> Result<()> {
            self.uart
                .driver_mut()
                .change_baudrate(baudrate.hertz())
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
            Ok(())
        }

        fn read_byte_blocking(&mut self) -> Result<u8> {
            let mut byte: [u8; 1] = [0];
            self.uart
                .driver()
                .borrow_mut()
                .read(&mut byte, 20)
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
            Ok(byte[0])
        }
    }
}

/// Doc shenanigans: stubs for ESP32 integration documentation when the ESP toolchain is not available
#[cfg(not(feature = "esp"))]
mod esp {
    // todo
}
