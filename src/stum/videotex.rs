use crate::stum::IntoSequence;
use num_enum::IntoPrimitive;

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntoPrimitive)]
pub enum C0 {
    NUL = 0x00,
    SOH = 0x01,
    EOL = 0x04,
    ENQ = 0x05,
    BEL = 0x07,
    /// Move cursor to the left
    BS = 0x08,
    /// Move cursor to the right
    HT = 0x09,
    /// Move the cursor down
    LF = 0x0A,
    /// Move the cursor up
    VT = 0x0B,
    /// Move the cursor at the first position of the first line and clear the screen
    /// Article separator
    FF = 0x0C,
    /// Move the cursor at the beginning of the line
    CR = 0x0D,
    SO = 0x0E,
    SI = 0x0F,
    DLE = 0x10,
    Con = 0x11,
    Rep = 0x12,
    Sep = 0x13,
    Coff = 0x14,
    NACK = 0x15,
    SYN = 0x16,
    CAN = 0x18,
    /// G2
    SS2 = 0x19,
    SUB = 0x1A,
    /// Call C1 control function
    ESC = 0x1B,
    SS3 = 0x1D,
    /// Move the cursor at the first position of the first line
    /// Article separator
    RS = 0x1E,
    /// Sub article separator
    US = 0x1F,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntoPrimitive, PartialEq, Eq)]
pub enum C1 {
    /// 0%
    CharBlack = 0x40,
    /// 50%
    CharRed = 0x41,
    /// 70%
    CharGreen = 0x42,
    /// 90%
    CharYellow = 0x43,
    /// 40%
    CharBlue = 0x44,
    /// 60%
    CharMagenta = 0x45,
    /// 80%
    CharCyan = 0x46,
    /// 100%
    CharWhite = 0x47,
    Blink = 0x48,
    Fixed = 0x49,
    NormalSize = 0x4C,
    DoubleHeight = 0x4D,
    DoubleWidth = 0x4E,
    DoubleSize = 0x4F,
    /// 0%
    BgBlack = 0x50,
    /// 50%
    BgRed = 0x51,
    /// 70%
    BgGreen = 0x52,
    /// 90%
    BgYellow = 0x53,
    /// 40%
    BgBlue = 0x54,
    /// 60%
    BgMagenta = 0x55,
    /// 80%
    BgCyan = 0x56,
    /// 100%
    BgWhite = 0x57,
    Mask = 0x58,
    StartLigning = 0x59,
    EndLigning = 0x5A,
    Csi = 0x5B,
    NormalBg = 0x5C,
    InvertBg = 0x5D,
    Unmask = 0x5F,

    EnqCursor = 0x61,
}

impl IntoSequence<2> for C1 {
    fn sequence(self) -> [u8; 2] {
        [C0::ESC.into(), self.into()]
    }
}

/// p. 103
#[repr(u8)]
#[derive(Debug, Clone, Copy, IntoPrimitive)]
pub enum G2 {
    Pound = 0x23,
    Dollar = 0x24,
    Hash = 0x26,
    Section = 0x27,
    LeftArrow = 0x2C,
    UpArrow = 0x2D,
    RightArrow = 0x2E,
    DownArrow = 0x2F,
    Degree = 0x30,
    PlusMinus = 0x31,
    Division = 0x38,
    OneQuarter = 0x3C,
    OneHalf = 0x3D,
    ThreeQuarters = 0x3E,
    Grave = 0x41,
    Acute = 0x42,
    Circumflex = 0x43,
    Diaeresis = 0x48,
    Cedille = 0x4B,
    OeMaj = 0x6A,
    OeMin = 0x7A,
    Beta = 0x7B,
}

impl IntoSequence<2> for G2 {
    fn sequence(self) -> [u8; 2] {
        [C0::SS2.into(), self.into()]
    }
}
