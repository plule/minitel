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

use videotex::{SIChar, Stroke, TouchesFonction, G0};

use std::io::{Error, ErrorKind, Result};

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

pub trait MinitelRead {
    /// Read a sequence of bytes from the minitel, blocking
    fn read(&mut self, data: &mut [u8]) -> Result<()>;
}

pub trait MinitelWrite {
    /// Send a sequence of bytes to the minitel
    fn send(&mut self, data: &[u8]) -> Result<()>;
    /// Flush the serial port
    fn flush(&mut self) -> Result<()>;
}

pub trait MinitelBaudrateControl {
    /// Change the baudrate of the serial port
    fn set_baudrate(&mut self, baudrate: Baudrate) -> Result<()>;
}

pub struct Minitel<S> {
    pub port: S,
}

impl<S> Minitel<S> {
    pub fn new(port: S) -> Self {
        Self { port }
    }
}

impl<S: MinitelRead + MinitelWrite> Minitel<S> {
    #[inline(always)]
    pub fn clear_screen(&mut self) -> Result<()> {
        self.write_byte(C0::FF)
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
}

impl<S: MinitelRead> Minitel<S> {
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

    /// Read a key stroke from the minitel assuming it is in S0 (text) mode.
    ///
    /// G0 and G2 characters are returned as unicode characters.
    pub fn read_s0_stroke(&mut self) -> Result<Stroke> {
        let b = self.read_byte()?;
        if let Ok(g0) = G0::try_from(b) {
            return Ok(Stroke::Char(g0.into()));
        }
        let c0 = C0::try_from(b).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        match c0 {
            C0::ESC => {
                // ESC code, C1 special char
                let c1 = C1::try_from(self.read_byte()?)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                Ok(Stroke::C1(c1))
            }
            C0::Sep => {
                // SEP code, function key
                let fct = TouchesFonction::try_from(self.read_byte()?)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                Ok(Stroke::Fonction(fct))
            }
            C0::SS2 => {
                // SS2 code, G2 char, returned as unicode char
                let g2 = G2::try_from(self.read_byte()?)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

                if let Some(diacritics) = g2.unicode_diacritic() {
                    // With diacritics, read one more byte for the base char
                    let char: char = self.read_byte()?.into();
                    let char = unicode_normalization::char::compose(char, diacritics).ok_or(
                        Error::new(ErrorKind::InvalidData, "Invalid diacritic composition"),
                    )?;
                    Ok(Stroke::Char(char))
                } else {
                    // Without diacritic, return the char directly
                    Ok(Stroke::Char(g2.char()))
                }
            }
            _ => Ok(Stroke::C0(c0)),
        }
    }

    #[inline(always)]
    pub fn wait_for(&mut self, byte: impl Into<u8> + Copy) -> Result<()> {
        for _ in 0..10 {
            if self.read_byte()? == byte.into() {
                return Ok(());
            }
        }
        Err(ErrorKind::TimedOut.into())
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

impl<S: MinitelWrite> Minitel<S> {
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
        if let Ok(c) = SIChar::try_from(c) {
            self.si_char(c)?;
            return Ok(());
        }
        Err(ErrorKind::InvalidData.into())
    }

    pub fn si_char(&mut self, c: SIChar) -> Result<()> {
        match c {
            SIChar::G0(g0) => self.write_byte(g0)?,
            SIChar::G0Diacritic(g0, g2) => {
                self.write_g2(g2)?;
                self.write_byte(g0)?;
            }
            SIChar::G2(g2) => self.write_g2(g2)?,
        }
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
}

/// Ability to communicate with a minitel through a serial port with baudrate control
impl<S: MinitelRead + MinitelWrite + MinitelBaudrateControl> Minitel<S> {
    pub fn search_speed(&mut self) -> Result<Baudrate> {
        for baudrate in [
            Baudrate::B1200,
            Baudrate::B300,
            Baudrate::B4800,
            Baudrate::B9600,
        ] {
            log::debug!("Trying baudrate: {}", baudrate);
            self.port.flush()?;
            self.port.set_baudrate(baudrate)?;
            //self.port.flush()?;
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

impl<T> MinitelRead for T
where
    T: std::io::Read,
{
    fn read(&mut self, data: &mut [u8]) -> Result<()> {
        self.read_exact(data)
    }
}

impl<T> MinitelWrite for T
where
    T: std::io::Write,
{
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.write_all(data)
    }

    fn flush(&mut self) -> Result<()> {
        self.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn read_stroke() {
        let seq: Vec<_> = "He?! ".bytes().collect();
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char('H'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char('e'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char('?'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char('!'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char(' '));

        let seq: Vec<_> = vec![0x20, 0x13, 0x41, 0x13, 0x49, 0x20, 0x1B, 0x54];
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char(' '));
        assert_eq!(
            minitel.read_s0_stroke().unwrap(),
            Stroke::Fonction(TouchesFonction::Envoi)
        );
        assert_eq!(
            minitel.read_s0_stroke().unwrap(),
            Stroke::Fonction(TouchesFonction::ConnexionFin)
        );
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char(' '));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::C1(C1::BgBlue));

        let seq: Vec<_> = vec![0x19, 0x42, 0x65, 0x19, 0x3D]; // SS2, ', e, SS2, ½
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char('é'));
        assert_eq!(minitel.read_s0_stroke().unwrap(), Stroke::Char('½'));
    }

    #[test]
    pub fn write_str() {
        let seq: Vec<u8> = Vec::new();
        let mut minitel = Minitel::new(std::io::Cursor::new(seq));
        minitel.write_str("Hé½").unwrap();
        let written = minitel.port.into_inner();
        assert_eq!(written, vec![0x48, 0x19, 0x42, 0x65, 0x19, 0x3D]); // H, SS2, ', e, SS2, ½
    }
}
