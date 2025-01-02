use std::io;

use layout::Flex;
use minitel::{
    stum::{
        videotex::{Stroke, TouchesFonction},
        Minitel, MinitelRead, MinitelWrite,
    },
    MinitelBackend,
};
use ratatui::{
    prelude::*,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        canvas::{Canvas, Map, MapResolution},
        Block, Paragraph, Tabs,
    },
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use style::Styled;
use symbols::{
    block,
    border::{
        self, QUADRANT_BOTTOM_HALF, QUADRANT_LEFT_HALF, QUADRANT_RIGHT_HALF,
        QUADRANT_TOP_LEFT_BOTTOM_LEFT_BOTTOM_RIGHT, QUADRANT_TOP_RIGHT_BOTTOM_LEFT_BOTTOM_RIGHT,
    },
};
use time::{Date, Duration, Month};
use tui_big_text::{BigText, PixelSize};

#[derive(Debug)]
pub struct App {
    selected_tab: SelectedTab,
    date: Date,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected_tab: SelectedTab::Calendrier,
            date: Date::from_calendar_date(2025, Month::January, 15).unwrap(),
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

        log::info!("Running the event loop");
        let loop_result = self.event_loop(terminal);
        log::info!("Event loop ended");
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
        log::info!("Entering event loop");
        while !self.exit {
            log::info!("Drawing frame");
            terminal.draw(|frame| self.draw(frame))?;
            log::info!("Handling events");
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
            log::info!("Received strocke {:?}", b);
            match b {
                Stroke::Fonction(TouchesFonction::Suite) => {
                    self.selected_tab = self.selected_tab.next()
                }
                Stroke::Fonction(TouchesFonction::Retour) => {
                    self.selected_tab = self.selected_tab.previous()
                }
                Stroke::Fonction(TouchesFonction::Sommaire) => self.exit = true,
                _ => match self.selected_tab {
                    SelectedTab::Calendrier => match b {
                        Stroke::Char('#') => {
                            self.date = self.date.saturating_add(Duration::days(20));
                            self.date = self.date.replace_day(15).unwrap();
                        }
                        Stroke::Char('*') => {
                            self.date = self.date.saturating_sub(Duration::days(20));
                            self.date = self.date.replace_day(15).unwrap();
                        }
                        _ => {}
                    },
                    SelectedTab::BigText => {}
                    SelectedTab::World => {}
                },
            }
        }
        Ok(())
    }
}

#[derive(Default, Clone, Copy, Debug, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Calendrier")]
    Calendrier,
    #[strum(to_string = "Big Text")]
    BigText,
    #[strum(to_string = "World")]
    World,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title_area, tabs_area, main_area, instructions_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        Paragraph::new(" Minitel App Example ")
            .style(Style::default().set_style((Color::Yellow, Color::Black)))
            .alignment(Alignment::Center)
            .render(title_area, buf);

        let titles = SelectedTab::iter().map(SelectedTab::title);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style((Color::Blue, Color::Yellow))
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(tabs_area, buf);

        Block::default()
            .style((Color::Blue, Color::Yellow))
            .render(main_area, buf);

        match self.selected_tab {
            SelectedTab::Calendrier => {
                let calendar_area =
                    center(main_area, Constraint::Length(24), Constraint::Length(9));
                Monthly::new(self.date, CalendarEventStore::default())
                    //.show_month_header(Style::default().bg(Color::Blue).fg(Color::White))
                    .show_weekdays_header(Style::default().fg(Color::Magenta))
                    .show_surrounding(Style::default().fg(Color::Cyan))
                    .block(
                        Block::bordered()
                            .border_set(QUADRANT_OUTSIDE_TOP_FULL)
                            .title(calendrier_title(self.date))
                            //.title_style((Color::White, Color::Blue))
                            .title_alignment(Alignment::Center)
                            .style((Color::Blue, Color::White)),
                    )
                    .render(calendar_area, buf);
            }
            SelectedTab::BigText => {
                BigText::builder()
                    .pixel_size(PixelSize::Sextant)
                    .style(Style::default().fg(Color::Green))
                    .lines(vec!["Hello".into(), "World!".into()])
                    .alignment(Alignment::Center)
                    .build()
                    .render(main_area, buf);
                //main_area_block.render(main_area, buf);
            }
            SelectedTab::World => {
                Canvas::default()
                    .block(Block::bordered().title("World"))
                    //.marker(self.marker)
                    .paint(|ctx| {
                        ctx.draw(&Map {
                            color: Color::Green,
                            resolution: MapResolution::High,
                        });
                    })
                    .x_bounds([-180.0, 180.0])
                    .y_bounds([-90.0, 90.0])
                    .render(main_area, buf);
            }
        }

        Block::default()
            .style((Color::Blue, Color::Cyan))
            .render(instructions_area, buf);
    }
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
        format!(" {self} ").fg(Color::Cyan).bg(Color::Red).into()
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub const QUADRANT_OUTSIDE_TOP_FULL: border::Set = border::Set {
    top_right: block::FULL,
    top_left: block::FULL,
    bottom_left: QUADRANT_TOP_LEFT_BOTTOM_LEFT_BOTTOM_RIGHT,
    bottom_right: QUADRANT_TOP_RIGHT_BOTTOM_LEFT_BOTTOM_RIGHT,
    vertical_left: QUADRANT_LEFT_HALF,
    vertical_right: QUADRANT_RIGHT_HALF,
    horizontal_top: block::FULL,
    horizontal_bottom: QUADRANT_BOTTOM_HALF,
};
