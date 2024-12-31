use backend::WindowSize;

use ratatui::backend::Backend;
use ratatui::prelude::*;
use symbols::line;

use minitel_stum::{
    videotex::{GrayScale, C0, C1},
    Minitel, MinitelRead, MinitelWrite,
};

/// Keep track of the contextual data
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharKind {
    None,
    /// Last char was a normal char
    Alphabet(char),
    /// Last char was a semi-graphic char
    SemiGraphic(u8),
}

impl CharKind {
    pub fn escape_code(&self) -> C0 {
        match self {
            CharKind::None => C0::NUL,
            CharKind::Alphabet(_) => C0::SI,
            CharKind::SemiGraphic(_) => C0::SO,
        }
    }
}

impl From<&str> for CharKind {
    fn from(c: &str) -> Self {
        match c {
            "▘" => CharKind::SemiGraphic(0x21),
            "▝" => CharKind::SemiGraphic(0x22),
            "▖" => CharKind::SemiGraphic(0x30),
            "▗" => CharKind::SemiGraphic(0x60),
            "▀" => CharKind::SemiGraphic(0x23),
            "▄" => CharKind::SemiGraphic(0x70),
            "▌" => CharKind::SemiGraphic(0x35),
            "▐" => CharKind::SemiGraphic(0x6A),
            "▙" => CharKind::SemiGraphic(0x75),
            "▛" => CharKind::SemiGraphic(0x37),
            "▜" => CharKind::SemiGraphic(0x6B),
            "▟" => CharKind::SemiGraphic(0x7A),
            "▚" => CharKind::SemiGraphic(0x64),
            "▞" => CharKind::SemiGraphic(0x26),
            "█" => CharKind::SemiGraphic(0x7F),
            "▉" => CharKind::SemiGraphic(0x7F),
            "▊" => CharKind::SemiGraphic(0x7F),
            "▋" => CharKind::SemiGraphic(0x35),
            "▍" => CharKind::SemiGraphic(0x35),
            "▎" => CharKind::SemiGraphic(0x20),
            "▏" => CharKind::SemiGraphic(0x20),
            "▇" => CharKind::SemiGraphic(0x7F),
            "▆" => CharKind::SemiGraphic(0x7C),
            "▅" => CharKind::SemiGraphic(0x7C),
            "▃" => CharKind::SemiGraphic(0x70),
            "▂" => CharKind::SemiGraphic(0x70),
            "▁" => CharKind::SemiGraphic(0x20),
            line::DOUBLE_HORIZONTAL => CharKind::SemiGraphic(0x73),
            line::DOUBLE_VERTICAL => CharKind::SemiGraphic(0x7F),
            line::DOUBLE_TOP_LEFT => CharKind::SemiGraphic(0x77),
            line::DOUBLE_TOP_RIGHT => CharKind::SemiGraphic(0x7B),
            line::DOUBLE_BOTTOM_LEFT => CharKind::SemiGraphic(0x77),
            line::DOUBLE_BOTTOM_RIGHT => CharKind::SemiGraphic(0x7B),

            /*"⠀" => CharKind::SemiGraphic(0x20),
            "⠁" => CharKind::SemiGraphic(0x21),
            "⠈" => CharKind::SemiGraphic(0x22),
            "⠉" => CharKind::SemiGraphic(0x23),
            "⠂" => CharKind::SemiGraphic(0x24),
            "⠃" => CharKind::SemiGraphic(0x25),
            "⠊" => CharKind::SemiGraphic(0x26),
            "⠋" => CharKind::SemiGraphic(0x27),
            "⠐" => CharKind::SemiGraphic(0x28),
            "⠢" => CharKind::SemiGraphic(0x29),



            "⠔" =>
            "⠠" => ,
            "⠰" => ,
            "⠑" => ,
            "⠡" => ,
            "⠱" => ,
            "⠒" => ,

            "⠲" => ,
            "⠓" => ,
            "⠣" => ,
            "⠳" => ,
            "⠄" => ,

            "⠤" => ,
            "⠴" => ,
            "⠅" => ,
            "⠕" => ,
            "⠥" => ,
            "⠵" => ,
            "⠆" => ,
            "⠖" => ,
            "⠦" => ,
            "⠶" => ,
            "⠇" => ,
            "⠗" => ,
            "⠧" => ,
            "⠷" => ,
            "⠘" => ,
            "⠨" => ,
            "⠸" => ,

            "⠙" => ,
            "⠩" => ,
            "⠹" => ,

            "⠚" => ,
            "⠪" => ,
            "⠺" => ,

            "⠛" => ,
            "⠫" => ,
            "⠻" => ,
            "⠌" => ,
            "⠜" => ,
            "⠬" => ,
            "⠼" => ,
            "⠍" => ,
            "⠝" => ,
            "⠭" => ,
            "⠽" => ,
            "⠎" => ,
            "⠞" => ,
            "⠮" => ,
            "⠾" => ,
            "⠏" => ,
            "⠟" => ,
            "⠯" => ,
            "⠿" => ,*/
            _ => CharKind::Alphabet(c.chars().next().unwrap()),
        }
    }
}

pub struct MinitelBackend<S: MinitelRead + MinitelWrite> {
    pub minitel: Minitel<S>,

    cursor_position: (u16, u16),
    last_char_kind: CharKind,
    char_attributes: Vec<C1>,
    zone_attributes: Vec<C1>,
}

impl<S: MinitelRead + MinitelWrite> MinitelBackend<S> {
    pub fn new(minitel: Minitel<S>) -> Self {
        Self {
            minitel,
            cursor_position: (255, 255),
            last_char_kind: CharKind::None,
            char_attributes: Vec::new(),
            zone_attributes: Vec::new(),
        }
    }
}

impl<S: MinitelRead + MinitelWrite> Backend for MinitelBackend<S> {
    #[inline(always)]
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        for (x, y, cell) in content {
            self.cursor_position.0 += 1;

            // Zone attributes: background color, invert, ...
            let zone_attributes = vec![match cell.bg {
                Color::Black => C1::BgBlack,
                Color::Red => C1::BgRed,
                Color::Green => C1::BgGreen,
                Color::Yellow => C1::BgYellow,
                Color::Blue => C1::BgBlue,
                Color::Magenta => C1::BgMagenta,
                Color::Cyan => C1::BgCyan,
                Color::Gray => GrayScale::Gray50.char(),
                Color::DarkGray => GrayScale::Gray40.char(),
                Color::LightRed => C1::BgRed,
                Color::LightGreen => C1::BgGreen,
                Color::LightYellow => C1::BgYellow,
                Color::LightBlue => C1::BgBlue,
                Color::LightMagenta => C1::BgMagenta,
                Color::LightCyan => C1::BgCyan,
                Color::White => C1::BgWhite,
                _ => C1::BgBlack,
            }];

            // Char attributes: foreground color, blink, ...
            let mut char_attributes = Vec::new();
            char_attributes.push(match cell.fg {
                Color::Black => C1::CharBlack,
                Color::Red => C1::CharRed,
                Color::Green => C1::CharGreen,
                Color::Yellow => C1::CharYellow,
                Color::Blue => C1::CharBlue,
                Color::Magenta => C1::CharMagenta,
                Color::Cyan => C1::CharCyan,
                Color::Gray => GrayScale::Gray50.char(),
                Color::DarkGray => GrayScale::Gray40.char(),
                Color::LightRed => C1::CharRed,
                Color::LightGreen => C1::CharGreen,
                Color::LightYellow => C1::CharYellow,
                Color::LightBlue => C1::CharBlue,
                Color::LightMagenta => C1::CharMagenta,
                Color::LightCyan => C1::CharCyan,
                Color::White => C1::CharWhite,
                _ => C1::CharWhite,
            });

            if cell.modifier.contains(Modifier::RAPID_BLINK)
                || cell.modifier.contains(Modifier::SLOW_BLINK)
            {
                char_attributes.push(C1::Blink);
            }

            // Chose between a char or a semi graphic
            let char_kind = CharKind::from(cell.symbol());

            // Check if the previous context is invalidated
            if self.cursor_position != (x, y)
                || std::mem::discriminant(&self.last_char_kind)
                    != std::mem::discriminant(&char_kind)
            {
                // Invalidated, we can start from scratch
                self.cursor_position = (x, y);
                self.char_attributes = Vec::new();
                self.zone_attributes = Vec::new();
                self.last_char_kind = char_kind;

                // Move the cursor to the right position, select the char set
                self.minitel.set_pos(x as u8, y as u8)?;
                self.minitel.write_byte(char_kind.escape_code() as u8)?;
            }

            match char_kind {
                CharKind::Alphabet(' ') => {
                    // Empty char, update the zone attributes if necessary
                    if self.zone_attributes != zone_attributes {
                        for attr in &zone_attributes {
                            self.minitel.write_c1(*attr)?;
                        }
                        self.zone_attributes.clone_from(&zone_attributes);
                    }
                    self.minitel.write_byte(0x20)?;
                }
                CharKind::Alphabet(c) => {
                    // Alphabetic char, update the char attributes if necessary
                    if self.char_attributes != char_attributes {
                        for attr in &char_attributes {
                            self.minitel.write_c1(*attr)?;
                        }
                        self.char_attributes.clone_from(&char_attributes);
                    }
                    self.minitel.write_char(c)?;
                }
                CharKind::SemiGraphic(c) => {
                    // Semigraphic char, update both the zone and char attributes if necessary
                    if self.zone_attributes != zone_attributes {
                        for attr in &zone_attributes {
                            self.minitel.write_c1(*attr)?;
                        }
                        self.zone_attributes.clone_from(&zone_attributes);
                    }
                    if self.char_attributes != char_attributes {
                        for attr in &char_attributes {
                            self.minitel.write_c1(*attr)?;
                        }
                        self.char_attributes.clone_from(&char_attributes);
                    }
                    // Write the semi graphic char
                    self.minitel.write_byte(c)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        self.minitel.hide_cursor()?;
        Ok(())
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        self.minitel.show_cursor()?;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> std::io::Result<ratatui::prelude::Position> {
        let (x, y) = self.minitel.get_pos()?;
        Ok(Position::new(x as u16, y as u16))
    }

    fn set_cursor_position<P: Into<ratatui::prelude::Position>>(
        &mut self,
        position: P,
    ) -> std::io::Result<()> {
        let position: Position = position.into();
        self.minitel.set_pos(position.x as u8, position.y as u8)?;
        Ok(())
    }

    fn clear(&mut self) -> std::io::Result<()> {
        self.minitel.clear_screen()?;
        Ok(())
    }

    fn size(&self) -> std::io::Result<ratatui::prelude::Size> {
        Ok(Size::new(40, 24))
    }

    fn window_size(&mut self) -> std::io::Result<ratatui::backend::WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size()?,
            pixels: self.size()?,
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.minitel.flush()?;
        Ok(())
    }
}
