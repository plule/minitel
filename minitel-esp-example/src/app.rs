use std::io;

use minitel::{
    stum::{
        protocol::{Baudrate, RoutingRx, RoutingTx},
        videotex::{Stroke, TouchesFonction},
    },
    ESPMinitel, ESPTerminal,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use symbols::border;

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
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
                    self.counter = self.counter.wrapping_add(1)
                }
                Stroke::Fonction(TouchesFonction::Retour) => {
                    self.counter = self.counter.wrapping_sub(1)
                }
                Stroke::Fonction(TouchesFonction::Sommaire) => self.exit = true,
                _ => {}
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
