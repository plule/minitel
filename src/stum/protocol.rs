//! Exchange protocol between its components
//!
//! Reference: <https://jbellue.github.io/stum1b/#2-6>

use core::fmt;
use std::fmt::{Display, Formatter};

use num_enum::{FromPrimitive, IntoPrimitive};

use crate::{
    stum::videotex::{self, C1},
    MinitelMessage,
};

/// Emission code of the Minitel modules
///
/// <https://jbellue.github.io/stum1b/#2-6-1>
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum RoutingTx {
    Screen = 0x50,
    Keyboard = 0x51,
    Modem = 0x52,
    Prise = 0x53,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Reception code of the Minitel modules
///
/// <https://jbellue.github.io/stum1b/#2-6-1>
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum RoutingRx {
    Screen = 0x58,
    Keyboard = 0x59,
    Modem = 0x5A,
    Prise = 0x5B,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Protocol messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolMessage {
    Pro1(Pro1),
    Pro2(Pro2, u8),
    Pro3(Pro3, u8, u8),
}

impl MinitelMessage for ProtocolMessage {
    fn message(self) -> Vec<u8> {
        match self {
            ProtocolMessage::Pro1(x) => {
                vec![videotex::C0::ESC.into(), C1::Pro1.into(), x.into()]
            }
            ProtocolMessage::Pro2(x, y) => {
                vec![videotex::C0::ESC.into(), C1::Pro2.into(), x.into(), y]
            }
            ProtocolMessage::Pro3(x, y, z) => {
                vec![videotex::C0::ESC.into(), C1::Pro3.into(), x.into(), y, z]
            }
        }
    }
}

impl ProtocolMessage {
    pub fn aiguillage(enable: bool, from: RoutingTx, to: RoutingRx) -> Self {
        ProtocolMessage::Pro3(
            if enable {
                Pro3::RoutingOn
            } else {
                Pro3::RoutingOff
            },
            to.into(),
            from.into(),
        )
    }

    pub fn set_speed(speed: Baudrate) -> Self {
        ProtocolMessage::Pro2(Pro2::Prog, speed.code())
    }

    pub fn function_mode(mode: FunctionMode, enable: bool) -> Self {
        ProtocolMessage::Pro2(if enable { Pro2::Start } else { Pro2::Stop }, mode.into())
    }
}

/// Sequence for a protocol message to enable or disable a routing
///
/// <https://jbellue.github.io/stum1b/#2-6-3>
pub fn aiguillage(enable: bool, from: RoutingTx, to: RoutingRx) -> ProtocolMessage {
    ProtocolMessage::Pro3(
        if enable {
            Pro3::RoutingOn
        } else {
            Pro3::RoutingOff
        },
        to.into(),
        from.into(),
    )
}

/// Protocol messages with one parameter
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum Pro1 {
    EnqSpeed = 0x74,
    /// <https://jbellue.github.io/stum1b/#2-6-6>
    EnqRom = 0x7B,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Protocol messages with two parameters
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum Pro2 {
    RoutingTo = 0x62,
    Start = 0x69,
    Stop = 0x6A,
    Prog = 0x6B,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Protocol messages with three parameters
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum Pro3 {
    RoutingOff = 0x60,
    RoutingOn = 0x61,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Protocol responses with two parameter
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum Pro2Resp {
    RepStatus = 0x73,
    QuerySpeedAnswer = 0x75,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Protocol responses with three parameter
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum Pro3Resp {
    RoutingFrom = 0x63,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Function mode for scrolling, error correcting, and lowercase
///
/// <https://jbellue.github.io/stum1b/#2-6-11>
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
pub enum FunctionMode {
    /// Mode Rouleau (screen scrolling)
    Rouleau = 0x43,
    /// PCE (Error Correcting Procedure)
    Procedure = 0x44,
    /// Minuscule (lowercase)
    Minuscule = 0x45,
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct RoutingStatus {
    pub prise: bool,
    pub modem: bool,
    pub keyboard: bool,
    pub screen: bool,
}

impl From<u8> for RoutingStatus {
    fn from(status: u8) -> Self {
        RoutingStatus {
            prise: status & 0b1000 != 0,
            modem: status & 0b0100 != 0,
            keyboard: status & 0b0010 != 0,
            screen: status & 0b0001 != 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Baudrate {
    B300,
    B1200,
    B4800,
    B9600,
}

impl Baudrate {
    pub fn hertz(&self) -> u32 {
        match self {
            Baudrate::B300 => 300,
            Baudrate::B1200 => 1200,
            Baudrate::B4800 => 4800,
            Baudrate::B9600 => 9600,
        }
    }

    pub fn code(&self) -> u8 {
        // P 1 E2 E1 E0 R2 R1 R0
        // P: Parity
        // E: Emission rate
        // R: Reception rate
        // 010 = 300 bauds
        // 100 = 1200 bauds
        // 110 = 4800 bauds
        // 111 = 9600 bauds
        // All the rates are symetrical (E = R)
        match self {
            Baudrate::B300 => 0b01_010_010,
            Baudrate::B1200 => 0b01_100_100,
            Baudrate::B4800 => 0b01_110_110,
            Baudrate::B9600 => 0b01_111_111,
        }
    }

    pub fn speeds() -> [Self; 4] {
        [
            Baudrate::B1200,
            Baudrate::B300,
            Baudrate::B4800,
            Baudrate::B9600,
        ]
    }
}

impl TryFrom<u8> for Baudrate {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b01_010_010 => Ok(Baudrate::B300),
            0b01_100_100 => Ok(Baudrate::B1200),
            0b01_110_110 => Ok(Baudrate::B4800),
            0b01_111_111 => Ok(Baudrate::B9600),
            _ => Err(value),
        }
    }
}

impl Display for Baudrate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} bauds", self.hertz())
    }
}

/// Content of the ROM retrived after PRO1 ENQROM
/// Are omitted the SOH and EOT starting and ending bytes
/// <https://jbellue.github.io/stum1b/#2-6-6>
pub struct Rom {
    pub manufacturer: u8,
    pub model: u8,
    pub version: u8,
}

impl From<[u8; 3]> for Rom {
    fn from(rom: [u8; 3]) -> Self {
        Rom {
            manufacturer: rom[0],
            model: rom[1],
            version: rom[2],
        }
    }
}
