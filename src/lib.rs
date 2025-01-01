pub mod toqst {
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
    };

    const MISTYPE_COLOR: Color = Color::Red;
    const UNTYPED_COLOR: Color = Color::Gray;
    const CORRECT_COLOR: Color = Color::Green;
    const MISTYPE_EXTRA_COLOR: Color = Color::Red;

    #[derive(Debug)]
    pub struct UserCursor {
        pub word_idx: usize,
        pub char_idx: usize,
        pub absolute_idx: usize,
        pub words: Vec<StyledWord>,
    }

    #[derive(Debug)]
    pub struct StyledWord {
        pub chars: Vec<StyledChar>,
        pub og_len: usize,
    }

    #[derive(Debug)]
    pub struct StyledChar {
        pub char: char,
        pub style: Style,
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
    }

    impl StyledWord {
        pub fn from_chars(chars: Vec<char>) -> Self {
            Self {
                og_len: chars.len(),
                chars: chars.into_iter().map(|ch| StyledChar::new(ch)).collect(),
            }
        }

        pub fn get_styled_word(&self) -> Line<'_> {
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

    impl UserCursor {
        pub fn handle_key_press(&mut self, pressed_char: char) {
            let word = self.words.get_mut(self.word_idx).unwrap();
            if let Some(ch) = word.get_mut_ch(self.char_idx) {
                if ch.char == ' ' {
                    todo!();
                }
                if ch.char == pressed_char {
                    ch.style = ch.style.fg(CORRECT_COLOR);
                } else {
                    ch.style = ch.style.fg(MISTYPE_COLOR);
                }
            } else {
                word.append_char(StyledChar::new_bad_char(pressed_char));
            }
            self.char_idx += 1;
            self.absolute_idx += 1;
        }

        pub fn handle_delete(&mut self) {
            if self.char_idx == 0 && self.word_idx == 0 {
                return;
            }

            assert_ne!(self.absolute_idx, 0);
            self.absolute_idx -= 1;

            let word = self.words.get_mut(self.word_idx).unwrap();
            self.char_idx -= 1;

            if self.char_idx >= word.og_len {
                word.chars.pop();
            } else {
                let ch = word.get_mut_ch(self.char_idx).unwrap();

                ch.style = ch.style.fg(UNTYPED_COLOR);
            }
        }
    }
}
