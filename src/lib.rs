#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod prelude {
    pub use crate::{
        AsyncMinitelBaudrateControl, AsyncMinitelRead, AsyncMinitelReadWrite,
        AsyncMinitelReadWriteBaudrate, AsyncMinitelWrite,
    };
}

/// Core Minitel types and traits
///
/// The stum module (Spécifications Techniques d'Utilisation du Minitel) exposes parts of the STUM1B specification.
pub mod stum;

/// Axum integration
///
/// Implements the necessary traits to use a Minitel terminal over an Axum websocket.
#[cfg(feature = "axum")]
pub mod axum;

/// Generic futures trait implementation
#[cfg(feature = "futures")]
pub mod futures;

/// ESP32 integration
///
/// Implements the necessary traits to use a Minitel terminal over an ESP32 microcontroller.
#[cfg(any(feature = "esp", feature = "espdoc"))]
pub mod esp;

/// Ratatui integration
///
/// Exposes a backend for ratatui, a terminal UI library. This helps writing interactive
/// applications for the Minitel.
#[cfg(feature = "ratatui")]
pub mod ratatui;

use std::io::{Error, ErrorKind, Result};

use stum::{
    protocol::{
        Baudrate, FunctionMode, Pro1, Pro2, Pro2Resp, Pro3Resp, ProtocolMessage, Rom, RoutingRx,
        RoutingTx,
    },
    videotex::{FunctionKey, UserInput, C0, C1, G0, G2},
};

pub trait MinitelMessage {
    fn message(self) -> Vec<u8>;
}

#[allow(async_fn_in_trait)]
pub trait AsyncMinitelRead {
    async fn read(&mut self, data: &mut [u8]) -> Result<()>;

    #[inline(always)]
    async fn read_byte(&mut self) -> Result<u8> {
        let mut data = [0];
        self.read(&mut data).await?;
        Ok(data[0])
    }

    /// Read a key stroke from the minitel assuming it is in S0 (text) mode.
    ///
    /// G0 and G2 characters are returned as unicode characters.
    async fn read_s0_stroke(&mut self) -> Result<UserInput> {
        let b = self.read_byte().await?;
        if let Ok(g0) = G0::try_from(b) {
            return Ok(UserInput::Char(g0.into()));
        }
        let c0 = C0::from(b);
        match c0 {
            C0::ESC => {
                // ESC code, C1 special char
                let c1 = C1::from(self.read_byte().await?);
                Ok(UserInput::C1(c1))
            }
            C0::Sep => {
                // SEP code, function key
                let fct = FunctionKey::try_from(self.read_byte().await?)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                Ok(UserInput::FunctionKey(fct))
            }
            C0::SS2 => {
                // SS2 code, G2 char, returned as unicode char
                let g2 = G2::try_from(self.read_byte().await?)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

                if let Some(diacritics) = g2.unicode_diacritic() {
                    // With diacritics, read one more byte for the base char
                    let char: char = self.read_byte().await?.into();
                    let char = unicode_normalization::char::compose(char, diacritics).ok_or(
                        Error::new(ErrorKind::InvalidData, "Invalid diacritic composition"),
                    )?;
                    Ok(UserInput::Char(char))
                } else {
                    // Without diacritic, return the char directly
                    Ok(UserInput::Char(g2.char()))
                }
            }
            _ => Ok(UserInput::C0(c0)),
        }
    }

    #[inline(always)]
    async fn wait_for(&mut self, byte: impl Into<u8> + Copy) -> Result<()> {
        for _ in 0..10 {
            if self.read_byte().await? == byte.into() {
                return Ok(());
            }
        }
        Err(ErrorKind::TimedOut.into())
    }

    #[inline(always)]
    async fn expect_read(&mut self, byte: impl Into<u8> + Copy) -> Result<()> {
        let got = self.read_byte().await?;
        if got != byte.into() {
            return Err(ErrorKind::InvalidData.into());
        }
        Ok(())
    }

    #[inline(always)]
    async fn read_pro2(&mut self, expected_ack: Pro2Resp) -> Result<u8> {
        self.wait_for(C0::ESC).await?;
        self.expect_read(C1::Pro2).await?;
        self.expect_read(expected_ack).await?;
        self.read_byte().await
    }

    #[inline(always)]
    async fn read_pro3(&mut self, expected_ack: Pro3Resp) -> Result<(u8, u8)> {
        self.wait_for(C0::ESC).await?;
        self.expect_read(C1::Pro3).await?;
        self.expect_read(expected_ack).await?;
        Ok((self.read_byte().await?, self.read_byte().await?))
    }
}

#[allow(async_fn_in_trait)]
pub trait AsyncMinitelWrite {
    async fn write(&mut self, data: &[u8]) -> Result<()>;
    async fn flush(&mut self) -> Result<()>;

    async fn send(&mut self, message: impl MinitelMessage) -> Result<()> {
        self.write(&message.message()).await
    }
}

/// Ability to change the baudrate of the serial port
#[allow(async_fn_in_trait)]
pub trait AsyncMinitelBaudrateControl {
    /// Change the baudrate of the serial port
    fn set_baudrate(&mut self, baudrate: Baudrate) -> Result<()>;

    /// Read, non async
    fn read_byte_blocking(&mut self) -> Result<u8>;
}

#[allow(async_fn_in_trait)]
pub trait AsyncMinitelReadWrite: AsyncMinitelRead + AsyncMinitelWrite {
    #[inline(always)]
    async fn read_rom(&mut self) -> Result<Rom> {
        self.send(ProtocolMessage::Pro1(Pro1::EnqRom)).await?;
        self.wait_for(C0::SOH).await?;
        let mut rom = [0; 3];
        self.read(&mut rom).await?;
        self.expect_read(C0::EOL).await?;
        Ok(rom.into())
    }

    #[inline(always)]
    async fn get_pos(&mut self) -> Result<(u8, u8)> {
        self.send(C1::EnqCursor).await?;
        self.wait_for(C0::US).await?;
        let mut position = [0; 2];
        self.read(&mut position).await?;
        Ok((position[1] - 0x40 - 1, position[0] - 0x40 - 1))
    }

    #[inline(always)]
    async fn set_function_mode(&mut self, mode: FunctionMode, enable: bool) -> Result<()> {
        self.send(ProtocolMessage::function_mode(mode, enable))
            .await?;
        let _status = self.read_pro2(Pro2Resp::RepStatus).await?;
        Ok(())
    }

    #[inline(always)]
    async fn set_routing(
        &mut self,
        enable: bool,
        recepter: RoutingRx,
        emitter: RoutingTx,
    ) -> Result<()> {
        self.send(ProtocolMessage::aiguillage(enable, emitter, recepter))
            .await?;
        let (_recepter, _status) = self.read_pro3(Pro3Resp::RoutingFrom).await?;
        Ok(())
    }

    #[inline(always)]
    async fn get_speed(&mut self) -> Result<Baudrate> {
        self.send(ProtocolMessage::Pro1(Pro1::EnqSpeed)).await?;
        let code = self.read_pro2(Pro2Resp::QuerySpeedAnswer).await?;
        Baudrate::try_from(code).map_err(|_| ErrorKind::InvalidData.into())
    }
}

/// Ability to communicate with a minitel through a serial port with baudrate control
#[allow(async_fn_in_trait)]
pub trait AsyncMinitelReadWriteBaudrate:
    AsyncMinitelReadWrite + AsyncMinitelBaudrateControl
{
    async fn search_speed(&mut self) -> Result<Baudrate> {
        for baudrate in [
            Baudrate::B1200,
            Baudrate::B9600,
            Baudrate::B300,
            Baudrate::B4800,
        ] {
            log::info!("Trying baudrate: {}", baudrate);
            self.flush().await?;
            self.set_baudrate(baudrate)?;
            self.send(ProtocolMessage::Pro1(Pro1::EnqSpeed)).await?;
            if let Ok(speed) = self.get_speed_blocking() {
                log::info!("Found baudrate: {}", speed);
                return Ok(speed);
            }
        }
        Err(ErrorKind::NotFound.into())
    }

    fn get_speed_blocking(&mut self) -> Result<Baudrate> {
        // blocking read, can't make async timeout work on esp
        for _ in 0..10 {
            if let Ok(C0::ESC) = self.read_byte_blocking().map(C0::from) {
                if let Ok(C1::Pro2) = self.read_byte_blocking().map(C1::from) {
                    if let Ok(Ok(Pro2Resp::QuerySpeedAnswer)) =
                        self.read_byte_blocking().map(Pro2Resp::try_from)
                    {
                        let code = self.read_byte_blocking()?;
                        return Baudrate::try_from(code).map_err(|_| ErrorKind::InvalidData.into());
                    }
                }
            }
        }
        Err(ErrorKind::NotFound.into())
    }

    #[inline(always)]
    async fn set_speed(&mut self, baudrate: Baudrate) -> Result<Baudrate> {
        self.send(ProtocolMessage::Pro2(Pro2::Prog, baudrate.code()))
            .await?;
        self.flush().await?;
        self.set_baudrate(baudrate)?;

        let speed_code = self.read_pro2(Pro2Resp::QuerySpeedAnswer).await?;
        let baudrate = Baudrate::try_from(speed_code).map_err(|_| ErrorKind::InvalidData)?;
        Ok(baudrate)
    }
}

impl<T> AsyncMinitelReadWrite for T where T: AsyncMinitelRead + AsyncMinitelWrite {}
impl<T> AsyncMinitelReadWriteBaudrate for T where
    T: AsyncMinitelRead + AsyncMinitelWrite + AsyncMinitelBaudrateControl
{
}

#[cfg(test)]
#[cfg(feature = "futures")]
mod tests {
    use ::futures::io::Cursor;
    use stum::videotex::StringMessage;

    use super::*;
    #[tokio::test]
    async fn read_stroke() {
        let seq: Vec<_> = "He?! ".bytes().collect();
        let mut minitel = Cursor::new(seq);
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char('H')
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char('e')
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char('?')
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char('!')
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char(' ')
        );

        let seq: Vec<_> = vec![0x20, 0x13, 0x41, 0x13, 0x49, 0x20, 0x1B, 0x54];
        let mut minitel = Cursor::new(seq);
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char(' ')
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::FunctionKey(FunctionKey::Envoi)
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::FunctionKey(FunctionKey::ConnexionFin)
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char(' ')
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::C1(C1::BgBlue)
        );

        let seq: Vec<_> = vec![0x19, 0x42, 0x65, 0x19, 0x3D]; // SS2, ', e, SS2, ½
        let mut minitel = Cursor::new(seq);
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char('é')
        );
        assert_eq!(
            minitel.read_s0_stroke().await.unwrap(),
            UserInput::Char('½')
        );
    }

    #[tokio::test]
    async fn write_str() {
        let seq: Vec<u8> = Vec::new();
        let mut minitel = Cursor::new(seq);
        minitel
            .send(StringMessage("Hé½".to_string()))
            .await
            .unwrap();
        let written = minitel.into_inner();
        assert_eq!(written, vec![0x48, 0x19, 0x42, 0x65, 0x19, 0x3D]); // H, SS2, ', e, SS2, ½
    }
}
