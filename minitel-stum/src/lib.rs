//! Spéfications Techniques d'Utilisation du Minitel
//!
//! This module defines the general constants extracted from the STUM1B specification.
//! Reference: https://jbellue.github.io/stum1b/

pub mod protocol;
pub mod videotex;

use crate::{
    protocol::{
        Baudrate, FunctionMode, Pro1, Pro2, Pro2Resp, Pro3, Pro3Resp, Protocol, Rom, RoutingRx,
        RoutingTx,
    },
    videotex::{C0, C1, G2},
};
use smallvec::SmallVec;
use unicode_normalization::UnicodeNormalization;

use std::io::{ErrorKind, Result};

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

pub trait SerialPort {
    /// Send a sequence of bytes to the minitel
    fn send(&mut self, data: &[u8]) -> Result<()>;
    /// Read a sequence of bytes from the minitel, blocking
    fn read(&mut self, data: &mut [u8]) -> Result<()>;
    /// Flush the serial port
    fn flush(&mut self) -> Result<()>;
}

pub trait BaudrateControl {
    /// Change the baudrate of the serial port
    fn set_baudrate(&mut self, baudrate: Baudrate) -> Result<()>;
}

pub struct Minitel<S: SerialPort> {
    port: S,
}

impl<S: SerialPort> Minitel<S> {
    pub fn new(port: S) -> Self {
        Self { port }
    }

    /// Write a raw sequence to the minitel
    #[inline(always)]
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        self.port.send(data)
    }

    /// Write a single byte to the minitel
    #[inline(always)]
    pub fn write_byte<T: Into<u8> + Copy>(&mut self, byte: T) -> Result<()> {
        self.write_bytes(&[byte.into()])
    }

    /// Read a raw sequence from the minitel
    #[inline(always)]
    pub fn read_bytes(&mut self, data: &mut [u8]) -> Result<()> {
        self.port.read(data)
    }

    /// Read a single byte from the minitel
    #[inline(always)]
    pub fn read_byte(&mut self) -> Result<u8> {
        let mut data = [0];
        self.read_bytes(&mut data)?;
        Ok(data[0])
    }

    /// Flush the serial port
    #[inline(always)]
    pub fn flush(&mut self) -> Result<()> {
        self.port.flush()
    }

    #[inline(always)]
    pub fn write_sequence<const N: usize>(&mut self, sequence: impl IntoSequence<N>) -> Result<()> {
        self.write_bytes(sequence.sequence().as_ref())
    }

    #[inline(always)]
    pub fn write_c1(&mut self, c1: C1) -> Result<()> {
        self.write_sequence(c1)
    }

    #[inline(always)]
    pub fn write_g2(&mut self, g2: G2) -> Result<()> {
        self.write_sequence(g2)
    }

    #[inline(always)]
    pub fn show_cursor(&mut self) -> Result<()> {
        self.write_byte(C0::Con)
    }

    #[inline(always)]
    pub fn hide_cursor(&mut self) -> Result<()> {
        self.write_byte(C0::Coff)
    }

    #[inline(always)]
    pub fn set_pos(&mut self, x: u8, y: u8) -> Result<()> {
        self.write_bytes(&[C0::US.into(), 0x40 + y, 0x40 + x + 1]) // allow access to y 0, not x 0// allow access to y 0, not x 0
    }

    #[inline(always)]
    pub fn cursor_down(&mut self) -> Result<()> {
        self.write_bytes(&[C0::LF.into()])
    }

    #[inline(always)]
    pub fn cursor_up(&mut self) -> Result<()> {
        self.write_byte(C0::VT)
    }

    #[inline(always)]
    pub fn cursor_right(&mut self) -> Result<()> {
        self.write_byte(C0::HT)
    }

    #[inline(always)]
    pub fn cursor_left(&mut self) -> Result<()> {
        self.write_byte(C0::BS)
    }

    #[inline(always)]
    pub fn start_zone(&mut self, funcs: &[C1]) -> Result<()> {
        for func in funcs {
            self.write_c1(*func)?;
        }
        self.zone_delimiter()?;
        Ok(())
    }

    #[inline(always)]
    pub fn zone_delimiter(&mut self) -> Result<()> {
        self.write_byte(0x20)
    }

    pub fn write_char(&mut self, c: char) -> Result<()> {
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
    pub fn write_str(&mut self, s: &str) -> Result<()> {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn writeln(&mut self, s: &str) -> Result<()> {
        let mut s = s.to_string();
        s.push_str("\r\n");
        self.write_str(&s)
    }

    #[inline(always)]
    pub fn clear_screen(&mut self) -> Result<()> {
        self.write_byte(C0::FF)
    }

    #[inline(always)]
    pub fn wait_for(&mut self, byte: impl Into<u8> + Copy) -> Result<()> {
        while self.read_byte()? != byte.into() {}
        Ok(())
    }

    #[inline(always)]
    pub fn expect_read(&mut self, byte: impl Into<u8> + Copy) -> Result<()> {
        let got = self.read_byte()?;
        if got != byte.into() {
            return Err(ErrorKind::InvalidData.into());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn read_rom(&mut self) -> Result<Rom> {
        self.pro1(Pro1::EnqRom)?;
        self.wait_for(C0::SOH)?;
        let mut rom = [0; 3];
        self.read_bytes(&mut rom)?;
        self.expect_read(C0::EOL)?;
        Ok(rom.into())
    }

    #[inline(always)]
    pub fn get_pos(&mut self) -> Result<(u8, u8)> {
        self.write_c1(C1::EnqCursor)?;
        self.wait_for(C0::US)?;
        let mut position = [0; 2];
        self.read_bytes(&mut position)?;
        Ok((position[1] - 0x40 - 1, position[0] - 0x40 - 1))
    }

    #[inline(always)]
    pub fn set_rouleau(&mut self, enable: bool) -> Result<()> {
        self.set_function_mode(FunctionMode::Rouleau, enable)
    }

    #[inline(always)]
    pub fn set_procedure(&mut self, enable: bool) -> Result<()> {
        self.set_function_mode(FunctionMode::Procedure, enable)
    }

    #[inline(always)]
    pub fn set_minuscule(&mut self, enable: bool) -> Result<()> {
        self.set_function_mode(FunctionMode::Minuscule, enable)
    }

    #[inline(always)]
    pub fn set_function_mode(&mut self, mode: FunctionMode, enable: bool) -> Result<()> {
        let start_stop = if enable { Pro2::Start } else { Pro2::Stop };
        self.pro2(start_stop, mode)?;
        let _status = self.read_pro2(Pro2Resp::RepStatus)?;
        Ok(())
    }

    #[inline(always)]
    pub fn set_routing(
        &mut self,
        enable: bool,
        recepter: RoutingRx,
        emitter: RoutingTx,
    ) -> Result<()> {
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
    pub fn get_speed(&mut self) -> Result<Baudrate> {
        self.pro1(Pro1::EnqSpeed)?;
        let code = self.read_pro2(Pro2Resp::QuerySpeedAnswer)?;
        Baudrate::try_from(code).map_err(|_| ErrorKind::InvalidData.into())
    }

    /// Protocol command with a single argument
    #[inline(always)]
    pub fn pro1(&mut self, x: Pro1) -> Result<()> {
        self.write_bytes(&Protocol::pro1(x))?;
        Ok(())
    }

    /// Protocol command with two arguments
    #[inline(always)]
    pub fn pro2(&mut self, x: Pro2, y: impl Into<u8> + Copy) -> Result<()> {
        self.write_bytes(&Protocol::pro2(x, y))?;
        Ok(())
    }

    /// Protocol command with three arguments
    #[inline(always)]
    pub fn pro3(
        &mut self,
        x: Pro3,
        y: impl Into<u8> + Copy,
        z: impl Into<u8> + Copy,
    ) -> Result<()> {
        self.write_bytes(&Protocol::pro3(x, y, z))?;
        Ok(())
    }

    #[inline(always)]
    pub fn read_pro2(&mut self, expected_ack: Pro2Resp) -> Result<u8> {
        self.wait_for(C0::ESC)?;
        self.expect_read(Protocol::Pro2)?;
        self.expect_read(expected_ack)?;
        self.read_byte()
    }

    #[inline(always)]
    pub fn read_pro3(&mut self, expected_ack: Pro3Resp) -> Result<(u8, u8)> {
        self.wait_for(C0::ESC)?;
        self.expect_read(Protocol::Pro3)?;
        self.expect_read(expected_ack)?;
        Ok((self.read_byte()?, self.read_byte()?))
    }
}

/// Ability to communicate with a minitel through a serial port with baudrate control
impl<S: SerialPort + BaudrateControl> Minitel<S> {
    pub fn search_speed(&mut self) -> Result<Baudrate> {
        for baudrate in [
            Baudrate::B1200,
            Baudrate::B300,
            Baudrate::B4800,
            Baudrate::B9600,
        ] {
            log::debug!("Trying baudrate: {}", baudrate);
            self.port.set_baudrate(baudrate)?;
            self.port.flush()?;
            if self.get_speed().is_ok() {
                log::debug!("Found baudrate: {}", baudrate);
                return Ok(baudrate);
            }
        }
        Err(ErrorKind::NotFound.into())
    }

    #[inline(always)]
    pub fn set_speed(&mut self, baudrate: Baudrate) -> Result<Baudrate> {
        self.pro2(Pro2::Prog, baudrate.code())?;
        self.port.flush()?;
        self.port.set_baudrate(baudrate)?;

        let speed_code = self.read_pro2(Pro2Resp::QuerySpeedAnswer)?;
        let baudrate = Baudrate::try_from(speed_code).map_err(|_| ErrorKind::InvalidData)?;
        Ok(baudrate)
    }
}

impl<S: SerialPort> From<S> for Minitel<S> {
    fn from(port: S) -> Self {
        Self::new(port)
    }
}
