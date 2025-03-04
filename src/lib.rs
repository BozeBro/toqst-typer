pub mod toqst {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::Span,
    };

    // User types the wrong letter when it should be another letter
    pub const MISTYPE_COLOR: Color = Color::Red;
    // Letter that has not been typed yet
    pub const UNTYPED_COLOR: Color = Color::Gray;
    // User types the correct letter (not a space)
    pub const CORRECT_COLOR: Color = Color::Green;
    // User types a letter when it should have been a space
    pub const MISTYPE_EXTRA_COLOR: Color = Color::Red;

    pub enum TypedState {
        Mistype,
        Untyped,
        Correct,
        MistypeExtra,
    }

    #[derive(Debug, Clone)]
    pub struct StyledWord {
        pub chars: Vec<StyledChar>, // A collection of chars that make up the word
        pub og_len: usize,          // Original length of the chars array
    }

    /// A Character that can be styled for TUI output
    /// Abstraction that Users type StyledChar (not char)
    #[derive(Debug, Clone)]
    pub struct StyledChar {
        char: char,
        style: Style,
    }

    impl StyledChar {
        pub fn new(ch: char) -> Self {
            Self {
                char: ch,
                style: Style::new().fg(UNTYPED_COLOR),
            }
        }
        // Create a Styled Character with a mistype connotation
        pub fn new_bad_char(ch: char) -> Self {
            Self {
                char: ch,
                style: Style::new().fg(MISTYPE_EXTRA_COLOR),
            }
        }

        // Switch the Styled State of a Styled Char
        pub fn switch_typed_state(&mut self, state: TypedState) {
            let color: Color;
            match state {
                TypedState::Mistype => color = MISTYPE_COLOR,
                TypedState::Untyped => color = UNTYPED_COLOR,
                TypedState::Correct => color = CORRECT_COLOR,
                TypedState::MistypeExtra => color = MISTYPE_EXTRA_COLOR,
            }
            self.style = self.style.fg(color);
        }

        pub fn get_char_data(&self) -> char {
            self.char
        }
    }

    impl StyledWord {
        pub fn from_chars(chars: Vec<char>) -> Self {
            Self {
                og_len: chars.len(),
                chars: chars.into_iter().map(|ch| StyledChar::new(ch)).collect(),
            }
        }

        pub fn from_string(chars: &str) -> Self {
            StyledWord::from_chars(chars.chars().collect())
        }

        pub fn get_styled_word(&self) -> Vec<Span<'_>> {
            self.chars
                .iter()
                .map(|char| Span::styled(String::from(char.char), char.style))
                .collect()
        }

        /// Get the Styled Representation of a word with a modifier added to one of the
        /// characters in the word
        /// It is assumed that the idx is within bounds of the word
        pub fn get_styled_with_modifier(&self, idx: usize, modifier: Modifier) -> Vec<Span<'_>> {
            self.chars
                .iter()
                .enumerate()
                .map(|(iter_idx, StyledChar { char, style })| {
                    let style = if iter_idx == idx {
                        style.add_modifier(modifier)
                    } else {
                        *style
                    };
                    Span::styled(String::from(*char), style)
                })
                .collect()
        }

        pub fn append_char(&mut self, ch: StyledChar) {
            self.chars.push(ch)
        }

        pub fn get_mut_ch(&mut self, index: usize) -> Option<&mut StyledChar> {
            self.chars.get_mut(index)
        }
    }
}
