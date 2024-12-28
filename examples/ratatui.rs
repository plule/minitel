use log::*;
use minitel::prelude::*;
use minitel_stum::videotex::{TouchesFonction, C0};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use symbols::border;

use std::{
    io,
    net::{TcpListener, TcpStream},
    thread::spawn,
};

use tungstenite::{accept, handshake::HandshakeRole, Error, HandshakeError, Result};

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut WebSocketMinitelTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut terminal.backend_mut().minitel)?;
        }
        terminal.backend_mut().minitel.clear_screen().unwrap();
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        log::info!("Rendering App");
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, minitel: &mut WebSocketMinitel<TcpStream>) -> io::Result<()> {
        if let Ok(C0::Sep) = C0::try_from(minitel.read_byte().unwrap()) {
            if let Ok(touche_fonction) = TouchesFonction::try_from(minitel.read_byte().unwrap()) {
                match touche_fonction {
                    TouchesFonction::Suite => self.counter = self.counter.saturating_add(1),
                    TouchesFonction::Retour => self.counter = self.counter.saturating_sub(1),
                    TouchesFonction::ConnexionFin => self.exit = true,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
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
            .border_set(border::QUADRANT_OUTSIDE);

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

fn handle_client(stream: TcpStream) -> Result<()> {
    info!("Running test");
    let socket = accept(stream).map_err(must_not_block)?;
    let mut minitel = WebSocketMinitel::new(socket);
    minitel.clear_screen().unwrap();
    let mut app = App::default();
    app.run(&mut WebSocketMinitelTerminal::new(MinitelBackend::new(minitel)).unwrap())?;
    Ok(())
}

fn main() {
    env_logger::init();

    let server = TcpListener::bind("0.0.0.0:3615").unwrap();

    for stream in server.incoming() {
        spawn(move || match stream {
            Ok(stream) => {
                if let Err(err) = handle_client(stream) {
                    match err {
                        Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                        e => error!("testéé: {}", e),
                    }
                }
            }
            Err(e) => error!("Error accepting stream: {}", e),
        });
    }
}

fn must_not_block<Role: HandshakeRole>(err: HandshakeError<Role>) -> Error {
    match err {
        HandshakeError::Interrupted(_) => panic!("Bug: blocking socket would block"),
        HandshakeError::Failure(f) => f,
    }
}
