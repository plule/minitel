use std::io;

use minitel::{
    stum::protocol::{Baudrate, RoutingRx, RoutingTx},
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
        terminal.clear()?;
        terminal.backend_mut().minitel.set_speed(Baudrate::B9600)?;
        terminal
            .backend_mut()
            .minitel
            .set_routing(false, RoutingRx::Modem, RoutingTx::Keyboard)?;
        let loop_result = self.event_loop(terminal);
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
        match minitel.read_byte()? {
            b'#' => {
                self.counter = self.counter.saturating_add(1);
            }
            b'*' => {
                self.counter = self.counter.saturating_sub(1);
            }
            b'0' => {
                self.exit = true;
            }
            _ => {}
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
