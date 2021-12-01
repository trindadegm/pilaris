use std::{
    cmp::Ordering,
    ops::Range,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Identifier,
    Keyword,
    ParensOpen,
    ParensClose,
    GroupBegin,
    GroupEnd,
    EOF,
}

enum ParseTokenState {
    NewLine,
    PopGroup,
    Identifier { ident_range: Range<usize> },
}

pub struct Lexer {
    input_filepath: PathBuf,
    code: String,
    parse_head: usize,
    last_token_range: Range<usize>,
    lineno: usize,
    columnno: usize,
    group_level: Vec<usize>,
    parse_token_state: ParseTokenState,
}

impl Lexer {
    #[inline]
    pub fn new(input_filepath: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Lexer::_new(input_filepath.as_ref())
    }

    fn _new(input_filepath: &Path) -> Result<Self, std::io::Error> {
        std::fs::read_to_string(input_filepath).map(|code| Lexer {
            input_filepath: input_filepath.to_path_buf(),
            code,
            parse_head: 0,
            last_token_range: 0..0,
            lineno: 1,
            columnno: 1,
            group_level: vec![],
            parse_token_state: ParseTokenState::NewLine,
        })
    }

    pub fn get_token(&mut self) -> Result<Token, LexicError> {
        use ParseTokenState::*;
        loop {
            if let Some(current_char) = self.code[self.parse_head..].chars().next() {
                match &self.parse_token_state {
                    NewLine => {
                        match current_char {
                            // Consider these whitespace
                            ' ' => (),
                            '\n' => self.newline(),
                            _ => {
                                let last_group_ident =
                                    self.group_level.last().cloned().unwrap_or(1);
                                match self.columnno.cmp(&last_group_ident) {
                                    Ordering::Greater => {
                                        self.group_level.push(self.columnno);
                                        self.parse_token_state = Identifier {
                                            ident_range: self.parse_head..self.parse_head,
                                        };
                                        break Ok(Token::GroupBegin);
                                    }
                                    Ordering::Less => {
                                        self.parse_token_state = PopGroup;
                                        continue;
                                    }
                                    _ => (),
                                }
                                self.parse_token_state = Identifier {
                                    ident_range: self.parse_head..self.parse_head,
                                };
                                continue;
                            }
                        }
                    }
                    PopGroup => {
                        self.group_level.pop();
                        let last_group_ident = self.group_level.last().cloned().unwrap_or(1);
                        match self.columnno.cmp(&last_group_ident) {
                            Ordering::Less => {
                                self.parse_token_state = PopGroup;
                                continue;
                            }
                            Ordering::Greater => {
                                return Err(LexicError::UnexpectedIdentationLevel {
                                    line: self.lineno,
                                    column: self.columnno,
                                    file: self.input_filepath.clone(),
                                })
                            }
                            _ => (),
                        }
                        self.parse_token_state = Identifier {
                            ident_range: self.parse_head..self.parse_head,
                        };
                    }
                    Identifier { ident_range } => match current_char {
                        c if (c.is_alphabetic()
                            || (c.is_alphanumeric() && self.parse_head != ident_range.start)) =>
                        {
                            self.parse_token_state = Identifier {
                                ident_range: ident_range.start..(self.parse_head + 1),
                            };
                        }
                        '\n' => {
                            self.newline();
                            self.parse_token_state = NewLine;
                        }
                        other => {
                            self.last_token_range = ident_range.start..ident_range.end;
                            self.head_forward(other);
                            self.parse_token_state = Identifier {
                                ident_range: self.parse_head..self.parse_head,
                            };
                            return Ok(Token::Identifier);
                        }
                    },
                } // End of match parse_token_state
                self.head_forward(current_char);
            } else {
                break Ok(Token::EOF);
            }
        } // End of loop
    } // End of fn get_token

    fn newline(&mut self) {
        self.lineno += 1;
        self.columnno = 0;
    }

    fn head_forward(&mut self, c: char) {
        self.parse_head += c.len_utf8();
        self.columnno += 1;
    }

    #[inline]
    pub fn token_str(&self) -> &str {
        &self.code[self.last_token_range.clone()]
    }
} // End of impl Lexer

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
