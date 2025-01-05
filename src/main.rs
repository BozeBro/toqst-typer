#![feature(iter_intersperse, file_buffered)]

extern crate chrono;
extern crate timer;

use std::{
    fs::File,
    io::{self, BufRead},
    vec,
};

use std::time::SystemTime;

use toqst_typer::toqst::*;

use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};

use rand::seq::IteratorRandom;

const SPEED_TYPING_TITLE: &'static str = "Toqst's Speed Typing Test";
const FILE: &'static str = "1000-most-common-words.txt";
const NUM_WORDS: usize = 50;
const EXTRA_CHAR_BOUNDARY: usize = 5;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let file = File::open_buffered(FILE)?;
    let words: Vec<_> = file
        .lines()
        .map(|line| line.unwrap_or(String::new()).trim().to_string())
        .filter(|word| !word.is_empty())
        .collect();

    // TODO: Should there be a restart option instead of only generating on startup
    let mut rng = rand::thread_rng();
    let rand_words = words.iter().choose_multiple(&mut rng, NUM_WORDS);

    let app_result = App::new(rand_words).run(terminal);
    ratatui::restore();
    // TODO: game loop so go to end game screen and give option to repeat
    println!("Game is done");
    app_result
}

#[derive(Debug)]
struct CursorWord {
    word: StyledWord,
    cursor_idx: usize,
}

#[derive(Debug)]
struct UserCursor {
    word_idx: usize,        // Position of the cursor in the word list
    words: Vec<CursorWord>, // Vector of words to type
}

/// Game Logic for the Speed Typing Test
///
/// Provides the cursor view into the Typing Test World
/// Responsible for moving the cursor positions and modifying the correct/incorrect colors when a
/// user types
impl UserCursor {
    fn is_game_done(&self) -> bool {
        self.word_idx == self.words.len()
    }
    fn handle_space_press(&mut self) {
        self.word_idx += 1;
    }

    fn get_cursor_word<'a>(&'a self) -> &'a CursorWord {
        &self.words[self.word_idx]
    }

    /// Style a word that is contained within the cursor word list
    /// It is assumed that each word is owened by the cursor and thus will live as long as the
    /// cursor
    /// It is assumed that each word is separated by a space word
    fn style_word<'a>(
        &'a self,
        idx: usize,
        CursorWord { word, cursor_idx }: &'a CursorWord,
    ) -> Vec<Span<'a>> {
        let cursor_word = self.get_cursor_word();

        let word_length = cursor_word.word.chars.len();

        // Each word is on an even index
        let cursor_in_word = idx == 2 * self.word_idx && *cursor_idx < word_length;
        // Each space is on an odd index
        let cursor_on_space = idx == 2 * self.word_idx + 1 && cursor_word.cursor_idx == word_length;

        assert!(
            cursor_word.cursor_idx <= word_length,
            "A cursor should be inside of the designated word or on the space after the word"
        );

        let cursor_modifier = Modifier::BOLD | Modifier::UNDERLINED;
        if cursor_in_word {
            return word.get_styled_with_modifier(*cursor_idx, cursor_modifier);
        } else if cursor_on_space {
            return vec![Span::styled(
                " ",
                Style::default().add_modifier(cursor_modifier),
            )];
        }
        word.get_styled_word()
    }

    fn handle_key_press(&mut self, pressed_char: char) {
        // implicit assumption that there is always a valid word that the cursor is on
        let CursorWord { word, cursor_idx } = self.words.get_mut(self.word_idx).unwrap();
        if let Some(ch) = word.get_mut_ch(*cursor_idx) {
            let data = ch.get_char_data();
            if data == pressed_char {
                ch.switch_typed_state(TypedState::Correct);
            } else {
                ch.switch_typed_state(TypedState::Mistype);
            }
        } else {
            if (word.chars.len() + 1) - word.og_len > EXTRA_CHAR_BOUNDARY {
                return;
            }
            // word.chars should always contains the original characters
            assert!(word.chars.len() >= word.og_len);
            word.append_char(StyledChar::new_bad_char(pressed_char));
        }
        *cursor_idx += 1;
    }

    /// User is attempting to delete a character from the type list
    /// The Cursor will not move/delete a character if at the very first character
    /// Keep the character in the word list if it belonged in the original word_list
    /// Otherwise, we must discard the character as if it never existed
    fn handle_delete(&mut self) {
        let cursor_idx = self.words.get(self.word_idx).unwrap().cursor_idx;

        if cursor_idx == 0 && self.word_idx == 0 {
            return;
        }

        // At the beginning of the word
        // Move to the previous word
        if cursor_idx == 0 {
            self.word_idx -= 1;
            return;
        }

        let CursorWord { word, cursor_idx } = self.words.get_mut(self.word_idx).unwrap();

        *cursor_idx -= 1;

        if *cursor_idx >= word.og_len {
            // Delete the extra character from the stream.
            // Not part of original word
            word.chars.pop();
        } else {
            // The character must still exist as we are under the word length
            word.get_mut_ch(*cursor_idx)
                .unwrap()
                .switch_typed_state(TypedState::Untyped);
        }
    }
}

#[derive(Debug)]
enum TypingEvent {
    AFK,
    TYPED(SystemTime),
}
/// Speed Typing Test Application
///
/// High Level Logic for Rendering the Terminal Typing Application onto Terminal
/// Receives different keypress events from users and forwards events to the Cursor to handle logic
struct App {
    user_typing: TypingEvent,
    should_exit: bool,
    cursor: UserCursor,
    layout: Layout,
}

impl App {
    // const TICK_RATE: Duration = Duration::from_secs(1);

    /// Create a new instance of the app.
    fn new(words: Vec<&String>) -> Self {
        let layout = Layout::vertical([Constraint::Percentage(100)]);
        Self {
            user_typing: TypingEvent::AFK,
            should_exit: false,
            cursor: UserCursor {
                word_idx: 0,
                words: words
                    .into_iter()
                    .map(|str| CursorWord {
                        word: StyledWord::from_string(&str),
                        cursor_idx: 0,
                    })
                    .collect(),
            },
            layout,
        }
    }

    /// Draw the entire terminal Application and position the cursor on the  screen
    fn draw(&self, frame: &mut Frame) {
        let rect = frame.area();

        frame.render_widget(self, rect);
    }

    fn is_typing_time_done(&self) -> bool {
        if let TypingEvent::TYPED(start_time) = self.user_typing {
            match start_time.elapsed() {
                Ok(elapsed) => {
                    return elapsed.as_secs() > 20;
                }
                Err(e) => {
                    panic!(
                        "Error for retrieving elapsed time has occurred with error: {}",
                        e
                    );
                }
            }
        }

        false
    }

    /// Run the app until the user exits.
    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;
            self.handle_events()?;
            // Because in immediate mode, we need to manually check the time ourselves
            // the precision will not be off by much if we manually check
            // Checking async is not too helpful because of these facts
            self.should_exit = self.cursor.is_game_done() || self.is_typing_time_done();
        }
        Ok(())
    }

    /// Handle events from the terminal.
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(ch) => {
                        if ch == ' ' {
                            self.cursor.handle_space_press();
                        } else {
                            self.cursor.handle_key_press(ch);
                        }
                        if matches!(self.user_typing, TypingEvent::AFK) {
                            self.user_typing = TypingEvent::TYPED(SystemTime::now());
                        }
                    }
                    KeyCode::Backspace | KeyCode::Delete => self.cursor.handle_delete(),
                    KeyCode::Esc => self.should_exit = true,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    /// Responsible for rendering just the Speed Typing test onto the screen and each of the
    /// words managed by the Cursor
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: There should be a timer countdown option
        // TODO: Scrolling on input would be nice
        let areas = self.layout.split(area);
        let separator = CursorWord {
            word: StyledWord::from_string(" "),
            cursor_idx: 0,
        };

        // Retrieve a vector of each word in Styled Form
        Paragraph::new(
            self.cursor
                .words
                .iter()
                .intersperse(&separator)
                .enumerate()
                .flat_map(|(idx, cursor_word)| self.cursor.style_word(idx, cursor_word))
                .collect::<Line<'_>>(),
        )
        .block(title_block(SPEED_TYPING_TITLE))
        .left_aligned()
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
