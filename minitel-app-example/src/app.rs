//! Application logic for the Minitel app example.

use std::io;

use minitel::{
    stum::{
        videotex::{FunctionKey, UserInput},
        Minitel, MinitelRead, MinitelWrite,
    },
    MinitelBackend,
};
use ratatui::{
    layout::Flex,
    prelude::*,
    style::Styled,
    symbols::border,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        canvas::{Canvas, Map, MapResolution},
        Block, Padding, Paragraph, Tabs, Wrap,
    },
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use time::{Date, Duration, Month};
use tui_big_text::{BigText, PixelSize};

/// Application state
#[derive(Debug)]
pub struct App {
    selected_tab: SelectedTab,
    date: Date,
    demo_disjoint: bool,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected_tab: SelectedTab::Bienvenue,
            date: Date::from_calendar_date(2025, Month::January, 15).unwrap(),
            demo_disjoint: false,
            exit: false,
        }
    }
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run<B: MinitelRead + MinitelWrite>(
        &mut self,
        terminal: &mut Terminal<MinitelBackend<B>>,
    ) -> io::Result<()> {
        log::info!("Running App");
        terminal.clear()?;

        let loop_result = self.event_loop(terminal);
        if let Err(err) = loop_result {
            log::error!("Error in event loop: {:?}", err);
        }
        terminal.clear()?;

        Ok(())
    }

    fn event_loop<B: MinitelRead + MinitelWrite>(
        &mut self,
        terminal: &mut Terminal<MinitelBackend<B>>,
    ) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut terminal.backend_mut().minitel)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events<B: MinitelRead + MinitelWrite>(
        &mut self,
        minitel: &mut Minitel<B>,
    ) -> io::Result<()> {
        if let Ok(b) = minitel.read_s0_stroke() {
            match b {
                UserInput::FunctionKey(FunctionKey::Suite) => {
                    self.selected_tab = self.selected_tab.next()
                }
                UserInput::FunctionKey(FunctionKey::Retour) => {
                    self.selected_tab = self.selected_tab.previous()
                }
                UserInput::FunctionKey(FunctionKey::Sommaire) => self.exit = true,
                _ => match self.selected_tab {
                    SelectedTab::Calendrier => match b {
                        UserInput::FunctionKey(FunctionKey::Correction) => {
                            self.date = self.date.saturating_add(Duration::days(20));
                            self.date = self.date.replace_day(15).unwrap();
                        }
                        UserInput::FunctionKey(FunctionKey::Annulation) => {
                            self.date = self.date.saturating_sub(Duration::days(20));
                            self.date = self.date.replace_day(15).unwrap();
                        }
                        _ => {}
                    },
                    SelectedTab::Borders | SelectedTab::World => {
                        if let UserInput::FunctionKey(FunctionKey::Envoi) = b {
                            self.demo_disjoint = !self.demo_disjoint;
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(())
    }
}

#[derive(Default, Clone, Copy, Debug, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Bienvenue")]
    Bienvenue,
    #[strum(to_string = "Cal")]
    Calendrier,
    #[strum(to_string = "Monde")]
    World,
    #[strum(to_string = "Bordures")]
    Borders,
}

impl Widget for &App {
    /// Draw the application to the ratatui buffer
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title_area, tabs_area, main_area, instructions_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        Paragraph::new(" Minitel App Example ")
            .style((Color::Yellow, Color::Black))
            .alignment(Alignment::Center)
            .render(title_area, buf);

        self.draw_tabs(buf, tabs_area, main_area);

        match self.selected_tab {
            SelectedTab::Bienvenue => {
                self.draw_welcome(buf, main_area);
            }
            SelectedTab::Calendrier => {
                self.draw_calendar(buf, main_area);
            }
            SelectedTab::World => {
                self.draw_world(buf, main_area);
            }
            SelectedTab::Borders => {
                self.draw_border_demo(buf, main_area);
            }
        }

        self.draw_instructions(buf, instructions_area);
    }
}

impl App {
    fn draw_tabs(&self, buf: &mut Buffer, tabs_area: Rect, main_area: Rect) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .select(selected_tab_index)
            .highlight_style(Style::default())
            .padding("", "")
            .divider(" ")
            .render(tabs_area, buf);

        Block::default()
            .bg(self.selected_tab.color())
            .render(main_area, buf);
    }

    fn draw_welcome(&self, buf: &mut Buffer, main_area: Rect) {
        let big_text_area = vcenter(main_area, Constraint::Length(10));
        BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .style(Style::default().set_style((Color::Blue, self.selected_tab.color())))
            .lines(vec![
                "Bienvenue".slow_blink().into(),
                "dans le".into(),
                "Minitel !".underlined().crossed_out().into(),
            ])
            .centered()
            .build()
            .render(big_text_area, buf);
    }

    fn draw_calendar(&self, buf: &mut Buffer, main_area: Rect) {
        let calendar_area = center(main_area, Constraint::Length(24), Constraint::Length(9));
        let calendar_block = Block::bordered()
            .border_set(QUADRANT_OUTSIDE_TOP_FULL)
            .title(calendrier_title(self.date))
            .title_alignment(Alignment::Center)
            .style((Color::Blue, Color::White));
        let [weekdays_area, days_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                .areas(calendar_block.inner(calendar_area));
        calendar_block.render(calendar_area, buf);
        Paragraph::new(" Di Lun Mar Mer Je Ve Sa ".fg(Color::Magenta).underlined())
            .render(weekdays_area, buf);
        Monthly::new(self.date, CalendarEventStore::default())
            .show_surrounding(Style::default().fg(Color::Cyan))
            .render(days_area, buf);
    }

    fn draw_world(&self, buf: &mut Buffer, main_area: Rect) {
        Canvas::default()
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Green,
                    resolution: MapResolution::High,
                });
            })
            .background_color(self.selected_tab.color())
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .render(main_area, buf);
        buf.set_style(main_area, Style::default().crossed_out());
        // Force semi-graphic mode
        if self.demo_disjoint {
            buf.set_style(main_area, Style::default().underlined());
        }
    }

    fn draw_border_demo(&self, buf: &mut Buffer, main_area: Rect) {
        let [h1, h2] = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .spacing(1)
            .margin(1)
            .areas(main_area);
        let [l11, l12, l13] = Layout::vertical([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .spacing(1)
        .areas(h1);

        let [l21, l22, l23] = Layout::vertical([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .spacing(1)
        .areas(h2);

        let mut border_style = Style::default();
        if self.demo_disjoint {
            border_style = border_style.underlined();
        }
        border_demo(
            " Full ",
            "Bordure pleine",
            border::FULL,
            border_style.set_style((Color::Black, Color::Green)),
        )
        .render(l11, buf);
        border_demo(
            " Quad Inside ",
            "Quadrants intérieur",
            border::QUADRANT_INSIDE,
            border_style.set_style((Color::Black, Color::Green)),
        )
        .render(l12, buf);
        border_demo(
            " Quad Outside ",
            "Quadrants extérieur",
            border::QUADRANT_OUTSIDE,
            border_style.set_style((Color::Black, Color::Cyan)),
        )
        .render(l13, buf);

        border_demo(
            " 8th Width ",
            "Largeur 1/8",
            border::ONE_EIGHTH_WIDE,
            border_style.set_style((Color::Black, Color::Green)),
        )
        .render(l21, buf);

        border_demo(
            " 8th Width bis ",
            "Largeur 1/8 décalée",
            minitel::ratatui::border::ONE_EIGHTH_WIDE_OFFSET,
            border_style.set_style((Color::Black, Color::Green)),
        )
        .render(l22, buf);

        border_demo(
            " beveled ",
            "Largeur 1/8 biseautée",
            minitel::ratatui::border::ONE_EIGHTH_WIDE_BEVEL,
            border_style.set_style((Color::Black, Color::Green)),
        )
        .render(l23, buf);
    }

    fn draw_instructions(&self, buf: &mut Buffer, instructions_area: Rect) {
        let instructions_1 = Line::from(vec![
            " Onglets:".into(),
            " Suite/Retour".reversed(),
            " Quitter:".into(),
            " Sommaire".reversed(),
        ]);

        let instructions_2 = match self.selected_tab {
            SelectedTab::Calendrier => {
                Line::from(vec![" Mois:".into(), " Correction/Annulation".reversed()])
            }
            SelectedTab::Borders | SelectedTab::World => {
                Line::from(vec![" Joint/Disjoint:".into(), " Envoi".reversed()])
            }
            _ => Line::default(),
        };

        Paragraph::new(vec![instructions_1, instructions_2])
            .style((Color::Yellow, Color::Blue))
            .render(instructions_area, buf);
    }
}

fn border_demo<'a>(
    name: &'a str,
    content: &'a str,
    border_set: border::Set,
    border_style: Style,
) -> Paragraph<'a> {
    let block = Block::bordered()
        .border_set(border_set)
        .border_style(border_style)
        .title_alignment(Alignment::Right)
        .title(name.set_style((Color::Yellow, Color::Black)))
        .padding(Padding::left(1));

    Paragraph::new(content)
        .style((Color::Blue, Color::Cyan))
        .wrap(Wrap { trim: false })
        .block(block)
}

fn calendrier_title(date: Date) -> Line<'static> {
    let month = match date.month() {
        Month::January => "Janvier",
        Month::February => "Février",
        Month::March => "Mars",
        Month::April => "Avril",
        Month::May => "Mai",
        Month::June => "Juin",
        Month::July => "Juillet",
        Month::August => "Août",
        Month::September => "Septembre",
        Month::October => "Octobre",
        Month::November => "Novembre",
        Month::December => "Décembre",
    };
    Line::from(vec![
        " < ".fg(Color::Green),
        format!("{} {}", month, date.year()).fg(Color::White),
        " > ".fg(Color::Green),
    ])
    .bg(Color::Blue)
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    /// Return tab's name as a styled `Line`
    fn title(self) -> Line<'static> {
        format!(" {self} ").fg(Color::Black).bg(self.color()).into()
    }

    fn color(self) -> Color {
        match self {
            SelectedTab::Calendrier => Color::Yellow,
            SelectedTab::Bienvenue => Color::Cyan,
            SelectedTab::World => Color::Magenta,
            SelectedTab::Borders => Color::Green,
        }
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn vcenter(area: Rect, vertical: Constraint) -> Rect {
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub const QUADRANT_OUTSIDE_TOP_FULL: border::Set = border::Set {
    top_right: "█",
    top_left: "█",
    bottom_left: "▙",
    bottom_right: "▟",
    vertical_left: "▌",
    vertical_right: "▐",
    horizontal_top: "█",
    horizontal_bottom: "▄",
};
