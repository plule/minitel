use crate::IntoSequence;
use bitvec::{order::Lsb0, view::BitView};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Virtual keystroke sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    // A single character, ASCII or G2
    Char(char),
    // A single control character
    C0(C0),
    // ESC C1 control character
    C1(C1),
    Fonction(TouchesFonction),
}

/// Base control characters
/// https://jbellue.github.io/stum1b/#2-2-1-2-4-2
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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

/// ESC control character
/// https://jbellue.github.io/stum1b/#2-2-1-2-4-2
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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

    /// Enquiry cursor position
    EnqCursor = 0x61,
}

impl IntoSequence<2> for C1 {
    fn sequence(self) -> [u8; 2] {
        [C0::ESC.into(), self.into()]
    }
}

/// Semi-graphic sextant characters
///
/// https://jbellue.github.io/stum1b/#2-2-1-2-8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct G1(pub u8);

impl From<G1> for u8 {
    fn from(g1: G1) -> u8 {
        g1.0
    }
}

impl G1 {
    // Sextant from the unicode Symbols for Legacy Computing (U+1FB0x...)
    // https://en.wikipedia.org/wiki/Symbols_for_Legacy_Computing
    // Some values are skipped (zero, full, vertical bars)...
    // To simplify, use braille as intermediate
    #[rustfmt::skip]
    const SEXTANT_TO_BRAILLE: [char; 60] = [
        '⠁', '⠈', '⠉', '⠂', '⠃', '⠊', '⠋', '⠐', '⠑', '⠘', '⠙', '⠒', '⠓', '⠚', '⠛', '⠄',
        '⠅', '⠌', '⠍', '⠆', '⠎', '⠏', '⠔', '⠕', '⠜', '⠝', '⠖', '⠗', '⠞', '⠟', '⠠', '⠡',
        '⠨', '⠩', '⠢', '⠣', '⠪', '⠫', '⠰', '⠱', '⠹', '⠲', '⠳', '⠺', '⠻', '⠤', '⠥', '⠬',
        '⠭', '⠦', '⠧', '⠮', '⠯', '⠴', '⠵', '⠼', '⠽', '⠶', '⠷', '⠾'
    ];

    pub fn new(val: u8) -> Self {
        G1(val)
    }

    /// Convert from the 3 rows of 2 bits into a G1 character
    pub fn from_bits(bits: [[bool; 2]; 3]) -> Self {
        let mut val: u8 = 0;
        val.view_bits_mut::<Lsb0>().set(0, bits[0][0]);
        val.view_bits_mut::<Lsb0>().set(1, bits[0][1]);
        val.view_bits_mut::<Lsb0>().set(2, bits[1][0]);
        val.view_bits_mut::<Lsb0>().set(3, bits[1][1]);
        val.view_bits_mut::<Lsb0>().set(4, bits[2][0]);
        val.view_bits_mut::<Lsb0>().set(5, true);
        val.view_bits_mut::<Lsb0>().set(6, bits[2][1]);
        G1(val)
    }

    /// Render the approximate semi graphic character matching the unicode value
    pub fn approximate_char(c: char) -> Option<Self> {
        let c = match c {
            // sextants: use braille as intermediate
            '\u{1FB00}'..='\u{1FB3C}' => Self::SEXTANT_TO_BRAILLE[c as usize - 0x1FB00],
            _ => c,
        };
        match c {
            // braille
            '\u{2800}'..'\u{2900}' => {
                let val = c as u32 - 0x2800;
                let mut bits = [[false; 2]; 3];
                bits[0][0] = val & 0b00000001 != 0;
                bits[1][0] = val & 0b00000010 != 0;
                bits[2][0] = val & 0b00000100 != 0;
                bits[0][1] = val & 0b00001000 != 0;
                bits[1][1] = val & 0b00010000 != 0;
                bits[2][1] = val & 0b00100000 != 0;
                Some(G1::from_bits(bits))
            }
            // quadrants
            '▘' => Some(G1(0x21)),
            '▝' => Some(G1(0x22)),
            '▖' => Some(G1(0x30)),
            '▗' => Some(G1(0x60)),
            '▀' => Some(G1(0x23)),
            '▄' => Some(G1(0x70)),
            '▌' => Some(G1(0x35)),
            '▐' => Some(G1(0x6A)),
            '▙' => Some(G1(0x75)),
            '▛' => Some(G1(0x37)),
            '▜' => Some(G1(0x6B)),
            '▟' => Some(G1(0x7A)),
            '▚' => Some(G1(0x64)),
            '▞' => Some(G1(0x26)),
            '█' => Some(G1(0x7F)),
            '▉' => Some(G1(0x7F)),
            '▊' => Some(G1(0x7F)),
            '▋' => Some(G1(0x35)),
            '▍' => Some(G1(0x35)),
            '▎' => Some(G1(0x20)),
            '▏' => Some(G1(0x20)),
            '▇' => Some(G1(0x7F)),
            '▆' => Some(G1(0x7C)),
            '▅' => Some(G1(0x7C)),
            '▃' => Some(G1(0x70)),
            '▂' => Some(G1(0x70)),
            '▁' => Some(G1(0x20)),
            _ => None,
        }
    }
}

/// https://jbellue.github.io/stum1b/#2-2-1-2-8
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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

impl G2 {
    pub fn char(self) -> char {
        match self {
            G2::Pound => '£',
            G2::Dollar => '$',
            G2::Hash => '#',
            G2::Section => '§',
            G2::LeftArrow => '←',
            G2::UpArrow => '↑',
            G2::RightArrow => '→',
            G2::DownArrow => '↓',
            G2::Degree => '°',
            G2::PlusMinus => '±',
            G2::Division => '÷',
            G2::OneQuarter => '¼',
            G2::OneHalf => '½',
            G2::ThreeQuarters => '¾',
            G2::Grave => '`',
            G2::Acute => '´',
            G2::Circumflex => '^',
            G2::Diaeresis => '¨',
            G2::Cedille => '¸',
            G2::OeMaj => 'Œ',
            G2::OeMin => 'œ',
            G2::Beta => 'β',
        }
    }

    pub fn unicode_diacritic(self) -> Option<char> {
        match self {
            G2::Grave => Some('\u{0300}'),
            G2::Acute => Some('\u{0301}'),
            G2::Circumflex => Some('\u{0302}'),
            G2::Diaeresis => Some('\u{0308}'),
            G2::Cedille => Some('\u{0327}'),
            _ => None,
        }
    }

    pub fn try_from_diactric(c: char) -> Option<Self> {
        match c {
            '\u{0300}' => Some(G2::Grave),
            '\u{0301}' => Some(G2::Acute),
            '\u{0302}' => Some(G2::Circumflex),
            '\u{0308}' => Some(G2::Diaeresis),
            '\u{0327}' => Some(G2::Cedille),
            _ => None,
        }
    }
}

impl TryFrom<char> for G2 {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '£' => Ok(G2::Pound),
            '$' => Ok(G2::Dollar),
            '#' => Ok(G2::Hash),
            '§' => Ok(G2::Section),
            '←' => Ok(G2::LeftArrow),
            '↑' => Ok(G2::UpArrow),
            '→' => Ok(G2::RightArrow),
            '↓' => Ok(G2::DownArrow),
            '°' => Ok(G2::Degree),
            '±' => Ok(G2::PlusMinus),
            '÷' => Ok(G2::Division),
            '¼' => Ok(G2::OneQuarter),
            '½' => Ok(G2::OneHalf),
            '¾' => Ok(G2::ThreeQuarters),
            //'`' => Ok(G2::Grave),
            //'´' => Ok(G2::Acute),
            //'^' => Ok(G2::Circumflex),
            //'¨' => Ok(G2::Diaeresis),
            //'¸' => Ok(G2::Cedille),
            'Œ' => Ok(G2::OeMaj),
            'œ' => Ok(G2::OeMin),
            'β' => Ok(G2::Beta),
            _ => Err(()),
        }
    }
}

/// Function keys, preceeded with C0::SEP
///
/// https://jbellue.github.io/stum1b/#2-3-6
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum TouchesFonction {
    Envoi = 0x41,
    Retour = 0x42,
    Repetition = 0x43,
    Guide = 0x44,
    Annulation = 0x45,
    Sommaire = 0x46,
    Correction = 0x47,
    Suite = 0x48,
    ConnexionFin = 0x49,
}

impl IntoSequence<2> for TouchesFonction {
    fn sequence(self) -> [u8; 2] {
        [C0::Sep.into(), self.into()]
    }
}

/// Convenience for black&white minitels
///
/// https://jbellue.github.io/stum1b/#1-3-2-4-3
pub enum GrayScale {
    Black,
    Gray40,
    Gray50,
    Gray60,
    Gray70,
    Gray80,
    Gray90,
    White,
}

impl GrayScale {
    pub fn char(&self) -> C1 {
        match self {
            GrayScale::Black => C1::CharBlack,
            GrayScale::Gray40 => C1::CharBlue,
            GrayScale::Gray50 => C1::CharRed,
            GrayScale::Gray60 => C1::CharMagenta,
            GrayScale::Gray70 => C1::CharGreen,
            GrayScale::Gray80 => C1::CharCyan,
            GrayScale::Gray90 => C1::CharYellow,
            GrayScale::White => C1::CharWhite,
        }
    }

    pub fn bg(&self) -> C1 {
        match self {
            GrayScale::Black => C1::BgBlack,
            GrayScale::Gray40 => C1::BgBlue,
            GrayScale::Gray50 => C1::BgRed,
            GrayScale::Gray60 => C1::BgMagenta,
            GrayScale::Gray70 => C1::BgGreen,
            GrayScale::Gray80 => C1::BgCyan,
            GrayScale::Gray90 => C1::BgYellow,
            GrayScale::White => C1::BgWhite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn semigraphic_from_bits() {
        assert_eq!(
            0x20,
            G1::from_bits([[false, false], [false, false], [false, false]]).0
        );
        assert_eq!(
            0x7F,
            G1::from_bits([[true, true], [true, true], [true, true]]).0
        );
        assert_eq!(
            0x2C,
            G1::from_bits([[false, false], [true, true], [false, false]]).0
        );
    }

    #[test]
    pub fn semigraphic_from_char() {
        assert_eq!(G1::approximate_char('⠉'), Some(G1(0x23)));
        assert_eq!(G1::approximate_char('⠯'), Some(G1(0x77)));
        assert_eq!(G1::approximate_char('⡯'), Some(G1(0x77)));
        assert_eq!(G1::approximate_char('⢯'), Some(G1(0x77)));
        assert_eq!(G1::approximate_char('⣯'), Some(G1(0x77)));
        assert_eq!(G1::approximate_char('⣿'), Some(G1(0x7F)));
        assert_eq!(G1::approximate_char('\u{1FB00}'), Some(G1(0x21)));
        assert_eq!(G1::approximate_char('\u{1FB28}'), Some(G1(0x6B)));
    }
}
