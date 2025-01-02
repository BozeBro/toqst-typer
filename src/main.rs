#![feature(iter_intersperse)]
use std::{io, vec};

use toqst_typer::toqst::*;

use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};

const SPEED_TYPING_TITLE: &'static str = "Toqst's Speed Typing Test";

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(vec![
        "variable",
        "function",
        "loop",
        "class",
        "object",
        "string",
        "list",
        "dictionary",
        "tuple",
        "set",
        "import",
        "module",
        "package",
        "exception",
        "try",
        "except",
        "return",
        "if",
        "else",
        "elif",
        "while",
        "for",
        "break",
        "continue",
        "pass",
        "lambda",
        "decorator",
        "generator",
        "yield",
        "recursion",
        "argument",
        "parameter",
        "scope",
        "namespace",
        "global",
        "local",
        "assert",
        "true",
        "false",
        "none",
        "identity",
        "equality",
        "operator",
        "importerror",
        "syntaxerror",
        "indentation",
        "comprehension",
        "slicing",
        "map",
        "filter",
    ])
    .run(terminal);
    ratatui::restore();
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
    fn handle_space_press(&mut self) {
        self.word_idx += 1;
    }

    fn get_cursor_word<'a>(&'a self) -> &'a CursorWord {
        &self.words[self.word_idx]
    }

    fn style_word<'a>(
        &'a self,
        idx: usize,
        CursorWord { word, cursor_idx }: &'a CursorWord,
    ) -> Vec<Span<'a>> {
        let cursor_word = self.get_cursor_word();
        let cursor_in_word = idx == 2 * self.word_idx && *cursor_idx < word.chars.len();
        let cursor_in_space =
            idx == 2 * self.word_idx + 1 && cursor_word.cursor_idx == cursor_word.word.chars.len();
        if cursor_in_word {
            return word
                .get_styled_with_modifier(*cursor_idx, Modifier::BOLD | Modifier::UNDERLINED);
        } else if cursor_in_space {
            return vec![Span::styled(
                " ",
                Style::default().add_modifier(Modifier::UNDERLINED),
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
            word.append_char(StyledChar::new_bad_char(pressed_char));
        }
        *cursor_idx += 1;
    }

    /// User is attempting to delete a character from the type list
    /// The Cursor will not move/delete a character if at the very first character
    /// Keep the character in the word list if it belonged in the original word_list
    /// Otherwise, we must discard the character as if it never existed
    fn handle_delete(&mut self) {
        let CursorWord {
            word: _,
            cursor_idx,
        } = self.words.get(self.word_idx).unwrap();

        if *cursor_idx == 0 && self.word_idx == 0 {
            return;
        }

        // At the beginning of the word
        // Move to the previous word
        if *cursor_idx == 0 {
            self.word_idx -= 1;
            return;
        }

        // reborrow with mutable perms
        let CursorWord { word, cursor_idx } = self.words.get_mut(self.word_idx).unwrap();

        *cursor_idx -= 1;

        if *cursor_idx >= word.og_len {
            // This is an extra character the User typed, must discard
            word.chars.pop();
        } else {
            // The character must exist as we are under the word length
            let ch = word.get_mut_ch(*cursor_idx).unwrap();

            ch.switch_typed_state(TypedState::Untyped);
        }
    }
}
/// Speed Typing Test Application
///
/// High Level Logic for Rendering the Terminal Typing Application onto Terminal
/// Receives different keypress events from users and forwards events to the Cursor to handle logic
#[derive(Debug)]
struct App {
    should_exit: bool,
    cursor: UserCursor,
    layout: Layout,
}

impl App {
    // const TICK_RATE: Duration = Duration::from_secs(1);

    /// Create a new instance of the app.
    fn new(words: Vec<&str>) -> Self {
        let layout = Layout::vertical([Constraint::Percentage(100)]);
        Self {
            should_exit: false,
            cursor: UserCursor {
                word_idx: 0,
                words: words
                    .into_iter()
                    .map(|str| CursorWord {
                        word: StyledWord::from_string(str),
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

    /// Run the app until the user exits.
    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Handle events from the terminal.
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(' ') => self.cursor.handle_space_press(),
                    KeyCode::Char(ch) => self.cursor.handle_key_press(ch),
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
