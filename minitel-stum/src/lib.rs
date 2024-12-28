//! Spéfications Techniques d'Utilisation du Minitel
//!
//! This module defines the general constants extracted from the STUM1B specification.
//! Reference: https://jbellue.github.io/stum1b/

pub mod protocol;
pub mod videotex;

use std::fmt::{self, Debug, Display, Formatter};

use crate::{
    protocol::{
        Baudrate, FunctionMode, Pro1, Pro2, Pro2Resp, Pro3, Pro3Resp, Protocol, Rom, RoutingRx,
        RoutingTx,
    },
    videotex::{C0, C1, G2},
};
use smallvec::SmallVec;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

/// Types that can be converted into a sequence of bytes in the
/// minitel serial protocol
pub trait IntoSequence<const N: usize> {
    /// Sequence of bytes, including the escape sequence
    fn sequence(self) -> [u8; N];
}

impl<T, const N: usize> IntoSequence<N> for &[T; N]
where
    T: Into<u8> + Copy,
{
    fn sequence(self) -> [u8; N] {
        self.map(|x| x.into())
    }
}

#[derive(Debug, Error)]
pub enum MinitelError {
    IOError(String),
    ProtocolError,
    UnknownBaudrate,
    FormattingError,
    Unimplemented,
}

impl Display for MinitelError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            MinitelError::IOError(e) => write!(f, "IO error: {}", e),
            MinitelError::ProtocolError => write!(f, "Protocol error"),
            MinitelError::UnknownBaudrate => write!(f, "Unknown baudrate"),
            MinitelError::FormattingError => write!(f, "Formatting error"),
            MinitelError::Unimplemented => write!(f, "Unimplemented"),
        }
    }
}

/// Ability to communicate with a minitel through a serial port
pub trait SerialMinitel {
    /// Send a sequence of bytes to the minitel
    fn send(&mut self, data: &[u8]) -> Result<(), MinitelError>;
    /// Read a sequence of bytes from the minitel, blocking
    fn read(&mut self, data: &mut [u8]) -> Result<(), MinitelError>;
    /// Flush the serial port
    fn flush(&mut self) -> Result<(), MinitelError>;

    /// Write a raw sequence to the minitel
    #[inline(always)]
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), MinitelError> {
        self.send(data)
    }

    #[inline(always)]
    fn write_byte<T: Into<u8> + Copy>(&mut self, byte: T) -> Result<(), MinitelError> {
        self.write_bytes(&[byte.into()])
    }

    /// Read a raw sequence from the minitel
    #[inline(always)]
    fn read_bytes(&mut self, data: &mut [u8]) -> Result<(), MinitelError> {
        self.read(data)
    }

    #[inline(always)]
    fn read_byte(&mut self) -> Result<u8, MinitelError> {
        let mut data = [0];
        self.read_bytes(&mut data)?;
        Ok(data[0])
    }

    #[inline(always)]
    fn write_sequence<const N: usize>(
        &mut self,
        sequence: impl IntoSequence<N>,
    ) -> Result<(), MinitelError> {
        self.write_bytes(sequence.sequence().as_ref())
    }

    #[inline(always)]
    fn write_c1(&mut self, c1: C1) -> Result<(), MinitelError> {
        self.write_sequence(c1)
    }

    #[inline(always)]
    fn write_g2(&mut self, g2: G2) -> Result<(), MinitelError> {
        self.write_sequence(g2)
    }

    #[inline(always)]
    fn show_cursor(&mut self) -> Result<(), MinitelError> {
        self.write_byte(C0::Con)
    }

    #[inline(always)]
    fn hide_cursor(&mut self) -> Result<(), MinitelError> {
        self.write_byte(C0::Coff)
    }

    #[inline(always)]
    fn set_pos(&mut self, x: u8, y: u8) -> Result<(), MinitelError> {
        self.write_bytes(&[C0::US.into(), 0x40 + y, 0x40 + x + 1]) // allow access to y 0, not x 0// allow access to y 0, not x 0
    }

    #[inline(always)]
    fn cursor_down(&mut self) -> Result<(), MinitelError> {
        self.write_bytes(&[C0::LF.into()])
    }

    #[inline(always)]
    fn cursor_up(&mut self) -> Result<(), MinitelError> {
        self.write_byte(C0::VT)
    }

    #[inline(always)]
    fn cursor_right(&mut self) -> Result<(), MinitelError> {
        self.write_byte(C0::HT)
    }

    #[inline(always)]
    fn cursor_left(&mut self) -> Result<(), MinitelError> {
        self.write_byte(C0::BS)
    }

    #[inline(always)]
    fn start_zone(&mut self, funcs: &[C1]) -> Result<(), MinitelError> {
        for func in funcs {
            self.write_c1(*func)?;
        }
        self.zone_delimiter()?;
        Ok(())
    }

    #[inline(always)]
    fn zone_delimiter(&mut self) -> Result<(), MinitelError> {
        self.write_byte(0x20)
    }

    fn write_char(&mut self, c: char) -> Result<(), MinitelError> {
        // ASCII, skip logic
        if c.is_ascii() {
            self.write_byte(c as u8)?;
            return Ok(());
        }

        // Specific case
        match c {
            'œ' => {
                self.write_g2(G2::OeMin)?;
                return Ok(());
            }
            'Œ' => {
                self.write_g2(G2::OeMaj)?;
                return Ok(());
            }
            'β' => {
                self.write_g2(G2::Beta)?;
                return Ok(());
            }
            '£' => {
                self.write_g2(G2::Pound)?;
                return Ok(());
            }
            '←' => {
                self.write_g2(G2::LeftArrow)?;
                return Ok(());
            }
            '↑' => {
                self.write_g2(G2::UpArrow)?;
                return Ok(());
            }
            '→' => {
                self.write_g2(G2::RightArrow)?;
                return Ok(());
            }
            '↓' => {
                self.write_g2(G2::DownArrow)?;
                return Ok(());
            }
            '°' => {
                self.write_g2(G2::Degree)?;
                return Ok(());
            }
            '±' => {
                self.write_g2(G2::PlusMinus)?;
                return Ok(());
            }
            '÷' => {
                self.write_g2(G2::Division)?;
                return Ok(());
            }
            '¼' => {
                self.write_g2(G2::OneQuarter)?;
                return Ok(());
            }
            '½' => {
                self.write_g2(G2::OneHalf)?;
                return Ok(());
            }
            '¾' => {
                self.write_g2(G2::ThreeQuarters)?;
                return Ok(());
            }
            _ => {}
        }

        // Diacritics
        let parts: SmallVec<[char; 2]> = c.nfd().take(2).collect();
        if let Some(c) = parts.get(1) {
            match *c as u32 {
                0x0300 => {
                    self.write_g2(G2::Grave)?;
                }
                0x0301 => {
                    self.write_g2(G2::Acute)?;
                }
                0x00EA => {
                    self.write_g2(G2::Circumflex)?;
                }
                0x0308 => {
                    self.write_g2(G2::Diaeresis)?;
                }
                0x0327 => {
                    self.write_g2(G2::Cedille)?;
                }
                _ => {}
            }
        }
        self.write_byte(parts[0] as u8)?;

        Ok(())
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> Result<(), MinitelError> {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn writeln(&mut self, s: &str) -> Result<(), MinitelError> {
        let mut s = s.to_string();
        s.push_str("\r\n");
        self.write_str(&s)
    }

    #[inline(always)]
    fn clear_screen(&mut self) -> Result<(), MinitelError> {
        self.write_byte(C0::FF)
    }

    #[inline(always)]
    fn wait_for(&mut self, byte: impl Into<u8> + Copy) -> Result<(), MinitelError> {
        while self.read_byte()? != byte.into() {}
        Ok(())
    }

    #[inline(always)]
    fn expect_read(&mut self, byte: impl Into<u8> + Copy) -> Result<(), MinitelError> {
        let got = self.read_byte()?;
        if got != byte.into() {
            return Err(MinitelError::ProtocolError);
        }
        Ok(())
    }

    #[inline(always)]
    fn read_rom(&mut self) -> Result<Rom, MinitelError> {
        self.pro1(Pro1::EnqRom)?;
        self.wait_for(C0::SOH)?;
        let mut rom = [0; 3];
        self.read_bytes(&mut rom)?;
        self.expect_read(C0::EOL)?;
        Ok(rom.into())
    }

    #[inline(always)]
    fn get_pos(&mut self) -> Result<(u8, u8), MinitelError> {
        self.write_c1(C1::EnqCursor)?;
        self.wait_for(C0::US)?;
        let mut position = [0; 2];
        self.read_bytes(&mut position)?;
        Ok((position[1] - 0x40 - 1, position[0] - 0x40 - 1))
    }

    #[inline(always)]
    fn set_rouleau(&mut self, enable: bool) -> Result<(), MinitelError> {
        self.set_function_mode(FunctionMode::Rouleau, enable)
    }

    #[inline(always)]
    fn set_procedure(&mut self, enable: bool) -> Result<(), MinitelError> {
        self.set_function_mode(FunctionMode::Procedure, enable)
    }

    #[inline(always)]
    fn set_minuscule(&mut self, enable: bool) -> Result<(), MinitelError> {
        self.set_function_mode(FunctionMode::Minuscule, enable)
    }

    #[inline(always)]
    fn set_function_mode(&mut self, mode: FunctionMode, enable: bool) -> Result<(), MinitelError> {
        let start_stop = if enable { Pro2::Start } else { Pro2::Stop };
        self.pro2(start_stop, mode)?;
        let _status = self.read_pro2(Pro2Resp::RepStatus)?;
        Ok(())
    }

    #[inline(always)]
    fn set_routing(
        &mut self,
        enable: bool,
        recepter: RoutingRx,
        emitter: RoutingTx,
    ) -> Result<(), MinitelError> {
        let cmd = if enable {
            Pro3::RoutingOn
        } else {
            Pro3::RoutingOff
        };
        self.pro3(cmd, recepter, emitter)?;
        let (_recepter, _status) = self.read_pro3(Pro3Resp::RoutingFrom)?;
        Ok(())
    }

    #[inline(always)]
    fn get_speed(&mut self) -> Result<Baudrate, MinitelError> {
        self.pro1(Pro1::EnqSpeed)?;
        let code = self.read_pro2(Pro2Resp::QuerySpeedAnswer)?;
        Baudrate::try_from(code).map_err(|_| MinitelError::UnknownBaudrate)
    }

    /// Protocol command with a single argument
    #[inline(always)]
    fn pro1(&mut self, x: Pro1) -> Result<(), MinitelError> {
        self.write_bytes(&Protocol::pro1(x))
    }

    /// Protocol command with two arguments
    #[inline(always)]
    fn pro2(&mut self, x: Pro2, y: impl Into<u8> + Copy) -> Result<(), MinitelError> {
        self.write_bytes(&Protocol::pro2(x, y))
    }

    /// Protocol command with three arguments
    #[inline(always)]
    fn pro3(
        &mut self,
        x: Pro3,
        y: impl Into<u8> + Copy,
        z: impl Into<u8> + Copy,
    ) -> Result<(), MinitelError> {
        self.write_bytes(&Protocol::pro3(x, y, z))
    }

    #[inline(always)]
    fn read_pro2(&mut self, expected_ack: Pro2Resp) -> Result<u8, MinitelError> {
        self.wait_for(C0::ESC)?;
        self.expect_read(Protocol::Pro2)?;
        self.expect_read(expected_ack)?;
        self.read_byte()
    }

    #[inline(always)]
    fn read_pro3(&mut self, expected_ack: Pro3Resp) -> Result<(u8, u8), MinitelError> {
        self.wait_for(C0::ESC)?;
        self.expect_read(Protocol::Pro3)?;
        self.expect_read(expected_ack)?;
        Ok((self.read_byte()?, self.read_byte()?))
    }
}

/// Ability to communicate with a minitel through a serial port with baudrate control
pub trait SerialPlugMinitel: SerialMinitel {
    /// Change the baudrate of the serial port
    fn internal_set_baudrate(&mut self, baudrate: Baudrate) -> Result<(), MinitelError>;

    fn change_baudrate(&mut self, baudrate: Baudrate) -> Result<(), MinitelError> {
        self.internal_set_baudrate(baudrate)
    }

    fn search_speed(&mut self) -> Result<Baudrate, MinitelError> {
        for baudrate in [
            Baudrate::B1200,
            Baudrate::B300,
            Baudrate::B4800,
            Baudrate::B9600,
        ] {
            log::debug!("Trying baudrate: {}", baudrate);
            self.internal_set_baudrate(baudrate)?;
            self.flush()?;
            if self.get_speed().is_ok() {
                log::debug!("Found baudrate: {}", baudrate);
                return Ok(baudrate);
            }
        }
        Err(MinitelError::UnknownBaudrate)
    }

    #[inline(always)]
    fn set_speed(&mut self, baudrate: Baudrate) -> Result<Baudrate, MinitelError> {
        self.pro2(Pro2::Prog, baudrate.code())?;
        self.flush()?;
        self.internal_set_baudrate(baudrate)?;

        let speed_code = self.read_pro2(Pro2Resp::QuerySpeedAnswer)?;
        let baudrate = Baudrate::try_from(speed_code).map_err(|_| MinitelError::UnknownBaudrate)?;
        Ok(baudrate)
    }
}
