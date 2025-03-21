use std::io::Write;

use backend::WindowSize;

use ratatui::prelude::*;
use ratatui::style::Styled;
use ratatui::{backend::Backend, buffer::Cell};

use crate::{
    stum::videotex::{GrayScale, Repeat, SIChar, SetPosition, C0, C1, G0, G1},
    MinitelMessage,
};

/// Keep track of the contextual data
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharKind {
    None,
    /// Last char was a normal char
    Alphabet(SIChar),
    /// Last char was a semi-graphic char
    SemiGraphic(G1),
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

/// Ratatui minitel backend
pub struct MinitelBackend<S: Write> {
    pub stream: S,

    cursor_position: (u16, u16),
    last_char_kind: CharKind,
    char_attributes: Vec<C1>,
    zone_attributes: Vec<C1>,
    repeat: u8,
    last_cell: Option<Cell>,
}

impl<S: Write> MinitelBackend<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            cursor_position: (255, 255),
            last_char_kind: CharKind::None,

            char_attributes: Vec::new(),
            zone_attributes: Vec::new(),
            repeat: 0,
            last_cell: None,
        }
    }

    fn send<T>(&mut self, message: T) -> std::io::Result<()>
    where
        T: MinitelMessage,
    {
        self.stream.write_all(&message.message())
    }
}

impl<S: Write> Backend for MinitelBackend<S> {
    #[inline(always)]
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            self.cursor_position.0 += 1;

            // Check if the cell is a repeat
            if (self.cursor_position.0, self.cursor_position.1) == (x, y)
                && Some(cell.to_owned()) == self.last_cell
            {
                self.repeat += 1;
                continue;
            } else if self.repeat > 0 {
                self.send(Repeat(self.repeat))?;
                self.repeat = 0;
            }
            self.last_cell = Some(cell.to_owned());

            // Zone attributes: background color, invert, ...
            let mut zone_attributes = vec![match cell.bg {
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
            zone_attributes.push(match cell.modifier.contains(Modifier::UNDERLINED) {
                true => C1::BeginUnderline,
                false => C1::EndUnderline,
            });
            zone_attributes.push(match cell.modifier.contains(Modifier::REVERSED) {
                true => C1::InvertBg,
                false => C1::NormalBg,
            });

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
            } else {
                char_attributes.push(C1::Fixed);
            }

            // Chose between a char or a semi graphic
            // The crossed out modifier is taken as prefering a semi graphic char
            let c = cell.symbol().chars().next().unwrap();
            let char_kind = if cell.modifier.contains(Modifier::CROSSED_OUT) {
                G1::approximate_char(c)
                    .map(CharKind::SemiGraphic)
                    .unwrap_or_else(|| {
                        SIChar::try_from(c)
                            .map(CharKind::Alphabet)
                            .unwrap_or(CharKind::None)
                    })
            } else {
                SIChar::try_from(c)
                    .map(CharKind::Alphabet)
                    .unwrap_or_else(|_| {
                        G1::approximate_char(c)
                            .map(CharKind::SemiGraphic)
                            .unwrap_or(CharKind::None)
                    })
            };

            // Check if the previous context is invalidated
            if self.cursor_position != (x, y)
                || std::mem::discriminant(&self.last_char_kind)
                    != std::mem::discriminant(&char_kind)
            {
                self.cursor_position = (x, y);
                self.char_attributes = Vec::new();
                self.zone_attributes = Vec::new();
                self.last_char_kind = char_kind;

                // Move the cursor to the right position, select the char set
                self.stream
                    .write_all(&SetPosition(x as u8, y as u8).message())?;

                self.send(char_kind.escape_code())?;
            }

            match char_kind {
                CharKind::Alphabet(SIChar::G0(G0(0x20))) => {
                    // Empty char, update the zone attributes if necessary
                    if self.zone_attributes != zone_attributes {
                        for attr in &zone_attributes {
                            self.send(*attr)?;
                        }
                        self.zone_attributes.clone_from(&zone_attributes);
                    }
                    self.send(SIChar::G0(G0(0x20)))?;
                }
                CharKind::Alphabet(c) => {
                    // Alphabetic char, update the char attributes if necessary
                    if self.char_attributes != char_attributes {
                        for attr in &char_attributes {
                            self.send(*attr)?;
                        }
                        self.char_attributes.clone_from(&char_attributes);
                    }
                    self.send(c)?;
                }
                CharKind::SemiGraphic(c) => {
                    // Semigraphic char, update both the zone and char attributes if necessary
                    if self.zone_attributes != zone_attributes {
                        for attr in &zone_attributes {
                            self.send(*attr)?;
                        }
                        self.zone_attributes.clone_from(&zone_attributes);
                    }
                    if self.char_attributes != char_attributes {
                        for attr in &char_attributes {
                            self.send(*attr)?;
                        }
                        self.char_attributes.clone_from(&char_attributes);
                    }
                    // Write the semi graphic char
                    self.send(c)?;
                }
                _ => {}
            }
        }
        if self.repeat > 0 {
            self.send(Repeat(self.repeat))?;
            self.repeat = 0;
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        self.send(C0::Coff)?;
        Ok(())
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        self.send(C0::Con)?;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> std::io::Result<ratatui::prelude::Position> {
        Ok(self.cursor_position.into())
    }

    fn set_cursor_position<P: Into<ratatui::prelude::Position>>(
        &mut self,
        position: P,
    ) -> std::io::Result<()> {
        let position: Position = position.into();
        self.send(SetPosition(position.x as u8, position.y as u8))?;
        Ok(())
    }

    fn clear(&mut self) -> std::io::Result<()> {
        self.send(C0::FF)?;
        Ok(())
    }

    fn size(&self) -> std::io::Result<ratatui::prelude::Size> {
        Ok(Size::new(40, 25))
    }

    fn window_size(&mut self) -> std::io::Result<ratatui::backend::WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size()?,
            pixels: self.size()?,
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub mod border {
    use ratatui::symbols::border;

    /// Variation on ONE_EIGHTH_WIDE offsetting it on the right to allow
    /// a consistent background transition in videotex mode.
    pub const ONE_EIGHTH_WIDE_OFFSET: border::Set = border::Set {
        top_right: "▁",
        top_left: " ",
        bottom_right: "▔",
        bottom_left: " ",
        vertical_left: "▕",
        vertical_right: "▕",
        horizontal_top: "▁",
        horizontal_bottom: "▔",
    };

    pub const ONE_EIGHTH_WIDE_BEVEL: border::Set = border::Set {
        top_right: "\\",
        top_left: "/",
        bottom_right: "/",
        bottom_left: "\\",
        vertical_left: "▏",
        vertical_right: "▕",
        horizontal_top: "▔",
        horizontal_bottom: "▁",
    };
}

pub trait StyledMinitelExt {
    type Item;
    #[cfg(feature = "invalidation-group")]
    fn invalidation_group(self, group: u8) -> Self::Item;
}

impl<T> StyledMinitelExt for T
where
    T: Styled<Item = T>,
{
    type Item = Self;
    #[cfg(feature = "invalidation-group")]
    fn invalidation_group(self, group: u8) -> Self::Item {
        let style = self.style().underline_color(Color::Indexed(group));
        self.set_style(style)
    }
}

pub mod widgets {
    use ratatui::{prelude::*, style::Styled};

    pub struct Fill {
        pub char: char,
        pub style: Style,
    }

    impl Default for Fill {
        fn default() -> Self {
            Self {
                char: '█',
                style: Style::default(),
            }
        }
    }

    impl Fill {
        pub fn with_char(self, char: char) -> Self {
            Self { char, ..self }
        }
    }

    impl Styled for Fill {
        type Item = Self;

        fn style(&self) -> Style {
            self.style
        }

        fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
            Self {
                style: style.into(),
                ..self
            }
        }
    }

    impl Widget for Fill {
        fn render(self, area: Rect, buf: &mut Buffer) {
            buf.set_style(area, self.style);
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(&self.char.to_string());
                    }
                }
            }
        }
    }
}
