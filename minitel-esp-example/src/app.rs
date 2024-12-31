use std::io;

use layout::Flex;
use minitel::{
    stum::{
        protocol::{Baudrate, RoutingRx, RoutingTx},
        videotex::{Stroke, TouchesFonction},
    },
    ESPMinitel, ESPTerminal,
};
use ratatui::{
    prelude::*,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        Block, Padding, Tabs,
    },
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use symbols::border;
use time::{Date, Duration, Month};

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
    pub fn run(&mut self, terminal: &mut ESPTerminal) -> io::Result<()> {
        log::info!("Running App");
        terminal.clear()?;
        terminal.backend_mut().minitel.set_speed(Baudrate::B9600)?;
        terminal
            .backend_mut()
            .minitel
            .set_routing(false, RoutingRx::Modem, RoutingTx::Keyboard)?;
        log::info!("Running the event loop");
        let loop_result = self.event_loop(terminal);
        log::info!("Event loop ended");
        if let Err(err) = loop_result {
            log::error!("Error in event loop: {:?}", err);
        }
        terminal.clear()?;
        terminal
            .backend_mut()
            .minitel
            .set_routing(true, RoutingRx::Modem, RoutingTx::Keyboard)?;
        Ok(())
    }

    fn event_loop(&mut self, terminal: &mut ESPTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut terminal.backend_mut().minitel)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, minitel: &mut ESPMinitel) -> io::Result<()> {
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
                    SelectedTab::Tab2 => {}
                    SelectedTab::Tab3 => {}
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
    #[strum(to_string = "T2")]
    Tab2,
    #[strum(to_string = "T3")]
    Tab3,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [no_area, mut title_area, main_area, instructions_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let titles = SelectedTab::iter().map(SelectedTab::title);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style((Color::Blue, Color::Yellow))
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(title_area, buf);

        Block::default()
            .style((Color::Blue, Color::Yellow))
            .render(main_area, buf);

        match self.selected_tab {
            SelectedTab::Calendrier => {
                let calendar_area =
                    center(main_area, Constraint::Length(23), Constraint::Length(10));
                Monthly::new(self.date, CalendarEventStore::default())
                    .show_month_header(Style::default().bg(Color::Blue).fg(Color::White))
                    .show_weekdays_header(Style::default().fg(Color::Magenta))
                    .show_surrounding(Style::default().fg(Color::Cyan))
                    .block(
                        Block::bordered()
                            .border_set(border::QUADRANT_INSIDE)
                            .fg(Color::Blue),
                    )
                    .render(calendar_area, buf);
            }
            SelectedTab::Tab2 => {
                //main_area_block.render(main_area, buf);
            }
            SelectedTab::Tab3 => {
                //main_area_block.render(main_area, buf);
            }
        }

        Block::default()
            .style((Color::Blue, Color::Cyan))
            .render(instructions_area, buf);
    }
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
        format!("  {self}  ").fg(Color::Cyan).bg(Color::Red).into()
    }

    /// A block surrounding the tab's content
    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style((Color::White, Color::Black))
    }

    /*const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Tab1 => tailwind::BLUE,
            Self::Tab2 => tailwind::EMERALD,
            Self::Tab3 => tailwind::INDIGO,
            Self::Tab4 => tailwind::RED,
        }
    }*/
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
