pub mod toqst {
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span, Text},
    };

    pub const MISTYPE_COLOR: Color = Color::Red;
    pub const UNTYPED_COLOR: Color = Color::Gray;
    pub const CORRECT_COLOR: Color = Color::Green;
    pub const MISTYPE_EXTRA_COLOR: Color = Color::Red;

    pub enum TypedState {
        Mistype,
        Untyped,
        Correct,
        MistypeExtra,
    }

    #[derive(Debug, Clone)]
    pub struct StyledWord {
        pub chars: Vec<StyledChar>,
        pub og_len: usize,
    }

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
        pub fn new_bad_char(ch: char) -> Self {
            Self {
                char: ch,
                style: Style::new().fg(MISTYPE_EXTRA_COLOR),
            }
        }

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
        pub fn append_char(&mut self, ch: StyledChar) {
            self.chars.push(ch);
        }

        pub fn get_mut_ch(&mut self, index: usize) -> Option<&mut StyledChar> {
            self.chars.get_mut(index)
        }
    }
}
