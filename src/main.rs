//! # [Ratatui] Paragraph example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use std::{
    io::{self},
    time::{Duration, Instant},
    vec,
};

use toqst_typer::toqst::*;

use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::Stylize,
    widgets::{Block, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};

const SPEED_TYPING_TITLE: &'static str = "Toqst's Speed Typing Test";

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
struct App {
    should_exit: bool,
    scroll: u16,
    last_tick: Instant,
    flag: bool,
    cursor: UserCursor,
    layout: Layout,
}

impl App {
    /// The duration between each tick.
    const TICK_RATE: Duration = Duration::from_secs(1);

    /// Create a new instance of the app.
    fn new() -> Self {
        let layout = Layout::vertical([Constraint::Percentage(100)]);
        Self {
            should_exit: false,
            scroll: 0,
            last_tick: Instant::now(),
            flag: false,
            cursor: UserCursor {
                word_idx: 0,
                char_idx: 0,
                absolute_idx: 0,
                words: vec![StyledWord::from_chars(vec!['a', 'b'])],
            },
            layout,
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let rect = frame.area();
        let [areas] = self.layout.areas(rect);
        let cursor_x = areas.x + self.cursor.absolute_idx as u16 + 1;
        let cursor_y = 1;
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        frame.render_widget(self, rect);
    }

    /// Run the app until the user exits.
    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;
            self.handle_events()?;
            if self.last_tick.elapsed() >= Self::TICK_RATE {
                self.on_tick();
                self.last_tick = Instant::now();
            }
        }
        Ok(())
    }

    /// Handle events from the terminal.
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if !(key.kind == KeyEventKind::Press) {
                return Ok(());
            }
            match key.code {
                KeyCode::Char(ch) => self.cursor.handle_key_press(ch),
                KeyCode::Backspace | KeyCode::Delete => self.cursor.handle_delete(),
                KeyCode::Esc => self.should_exit = true,
                _ => {}
            }
        }
        Ok(())
    }

    /// Update the app state on each tick.
    fn on_tick(&mut self) {
        self.scroll = (self.scroll + 1) % 10;
        self.flag = !self.flag;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = self.layout.split(area);
        Paragraph::new(
            self.cursor
                .words
                .iter()
                .map(|word| word.get_styled_word())
                .collect::<Vec<_>>(),
        )
        .block(title_block(SPEED_TYPING_TITLE))
        .wrap(Wrap { trim: true })
        .render(areas[0], buf);
    }
}

/// Create a bordered block with a title.
fn title_block(title: &str) -> Block {
    Block::bordered()
        .gray()
        .title(title.bold().into_centered_line())
}
