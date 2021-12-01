use std::{
    io,
    ops::Range,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Identifier,
    Keyword,
    Colon,
    ParensOpen,
    ParensClose,
    GroupBegin,
    GroupEnd,
    EOF,
}

#[derive(Clone, Debug)]
pub enum State {
    Looking,
    AccIdent { range: Range<usize> },
}

pub struct Lexer {
    input_filepath: PathBuf,
    code: String,
    current_line: usize,
    current_column: usize,
    input_head: usize,
    state: State,
    token_range: Range<usize>,
}

impl Lexer {
    const IDENT_BREAKERS: [char; 4] = [' ', '\n', '(', ')'];
    const WHITESPACE: [char; 2] = [' ', '\n'];

    /// Creates a new lexer for a source file
    #[inline]
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::_new(path.as_ref())
    }

    fn _new(path: &Path) -> io::Result<Self> {
        std::fs::read_to_string(path).map(|code| {
            let input_filepath = path.to_path_buf();
            Self {
                code,
                input_filepath,
                current_line: 0,
                current_column: 0,
                input_head: 0,
                state: State::Looking,
                token_range: 0..0,
            }
        })
    }

    pub fn get_token(&mut self) -> Result<Token, LexicError> {
        self.token_range = 0..0;
        loop {
            let current_c = if let Some(c) = self.getc() {
                c
            } else {
                break Ok(Token::EOF);
            };

            match self.state.clone() {
                State::Looking => match current_c {
                    c if c.is_alphabetic() => {
                        self.state = State::AccIdent {
                            range: self.input_head..(self.input_head + c.len_utf8()),
                        };
                        self.advance();
                    }
                    '(' => {
                        self.token_range =
                            self.input_head..(self.input_head + current_c.len_utf8());
                        self.state = State::Looking;
                        self.advance();
                        break Ok(Token::ParensOpen);
                    }
                    ')' => {
                        self.token_range =
                            self.input_head..(self.input_head + current_c.len_utf8());
                        self.state = State::Looking;
                        self.advance();
                        break Ok(Token::ParensClose);
                    }
                    ':' => {
                        self.token_range =
                            self.input_head..(self.input_head + current_c.len_utf8());
                        self.state = State::Looking;
                        self.advance();
                        break Ok(Token::Colon);
                    }
                    c if Self::WHITESPACE.contains(&c) => {
                        self.advance();
                    }
                    _ => break Err(self.err_unexpected_char(current_c)),
                },
                State::AccIdent { range } => match current_c {
                    c if c.is_alphanumeric() => {
                        self.advance();
                        self.state = State::AccIdent {
                            range: range.start..self.input_head,
                        };
                    }
                    c if Self::IDENT_BREAKERS.contains(&c) => {
                        self.token_range = range;
                        self.state = State::Looking;
                        break Ok(Token::Identifier);
                    }
                    _ => break Err(self.err_unexpected_char(current_c)),
                },
            }
        }
    }

    #[inline]
    pub fn getc(&self) -> Option<char> {
        self.code[self.input_head..].chars().next()
    }

    pub fn advance(&mut self) {
        let c = self.getc();
        let char_length = c.map(char::len_utf8).unwrap_or(0);
        self.input_head += char_length;
        match c {
            Some('\n') => {
                self.current_line += 1;
                self.current_column = 0;
            }
            Some(_) => self.current_column += 1,
            None => (),
        }
    }

    #[inline]
    pub fn input_filepath(&self) -> &Path {
        &self.input_filepath
    }

    #[inline]
    pub fn token_str(&self) -> &str {
        &self.code[self.token_range.clone()]
    }

    pub fn token_start_column(&self) -> usize {
        self.current_column - self.token_str().chars().count()
    }

    fn err_unexpected_char(&self, c: char) -> LexicError {
        LexicError::UnexpectedCharacter {
            c,
            file: self.input_filepath.clone(),
            line: self.current_line + 1,
            column: self.current_column + 1,
        }
    }
}

#[derive(Debug)]
pub enum LexicError {
    UnexpectedCharacter {
        c: char,
        file: PathBuf,
        line: usize,
        column: usize,
    },
    UnexpectedIdentationLevel {
        file: PathBuf,
        line: usize,
        column: usize,
    },
}

use std::error::Error;
use std::fmt::{Display, Formatter};

impl Error for LexicError {}
impl Display for LexicError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use LexicError::*;
        match self {
            UnexpectedCharacter {
                c,
                file,
                line,
                column,
            } => {
                write!(
                    f,
                    "{}:{}: Unexpected character '{}' at column {}",
                    file.display(),
                    line,
                    c,
                    column
                )
            }
            UnexpectedIdentationLevel {
                file,
                line,
                column: _,
            } => {
                write!(
                    f,
                    "{}:{}: Unexpected identation level at line {}",
                    file.display(),
                    line,
                    line,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GET_TOKEN_LIMIT: usize = 1000;

    #[test]
    fn simple_1() {
        let mut lexer = Lexer::new("input_examples/simple1.plr").unwrap();
        for _ in 0..GET_TOKEN_LIMIT {
            let tok = lexer.get_token().unwrap();
            println!("{}", lexer.token_str());
            if tok == Token::EOF {
                break;
            }
        }
    }
}
