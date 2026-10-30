use crate::diagnostics::KodeError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),

    Print,
    True,
    False,

    Plus,
    Minus,
    Star,
    Slash,

    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    LeftParen,
    RightParen,

    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

pub fn lex(source: &str) -> Result<Vec<Token>, KodeError> {
    Lexer::new(source).scan()
}

struct Lexer {
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    fn scan(mut self) -> Result<Vec<Token>, KodeError> {
        while !self.is_at_end() {
            let ch = self.advance();

            match ch {
                ' ' | '\t' | '\r' => {}

                '\n' => {
                    self.add(TokenKind::Newline);
                    self.line += 1;
                    self.column = 1;
                }

                '#' => {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }

                '(' => self.add(TokenKind::LeftParen),
                ')' => self.add(TokenKind::RightParen),

                '+' => self.add(TokenKind::Plus),
                '-' => self.add(TokenKind::Minus),
                '*' => self.add(TokenKind::Star),
                '/' => self.add(TokenKind::Slash),

                '=' => {
                    if self.match_char('=') {
                        self.add(TokenKind::EqualEqual);
                    } else {
                        self.add(TokenKind::Equal);
                    }
                }

                '!' => {
                    if self.match_char('=') {
                        self.add(TokenKind::BangEqual);
                    } else {
                        return Err(self.error("expected '=' after '!'"));
                    }
                }

                '<' => {
                    if self.match_char('=') {
                        self.add(TokenKind::LessEqual);
                    } else {
                        self.add(TokenKind::Less);
                    }
                }

                '>' => {
                    if self.match_char('=') {
                        self.add(TokenKind::GreaterEqual);
                    } else {
                        self.add(TokenKind::Greater);
                    }
                }

                '"' => self.scan_string()?,

                c if c.is_ascii_digit() => self.scan_number(c)?,

                c if c.is_ascii_alphabetic() || c == '_' => {
                    self.scan_identifier(c);
                }

                other => {
                    return Err(self.error(&format!(
                        "unexpected character '{}'",
                        other
                    )));
                }
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
            column: self.column,
        });

        Ok(self.tokens)
    }

    fn scan_identifier(&mut self, first: char) {
        let start = self.column - 1;
        let mut name = String::from(first);

        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            name.push(self.advance());
        }

        let kind = match name.as_str() {
            "print" => TokenKind::Print,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(name),
        };

        self.tokens.push(Token {
            kind,
            line: self.line,
            column: start,
        });
    }

    fn scan_number(&mut self, first: char) -> Result<(), KodeError> {
        let start = self.column - 1;
        let mut number = String::from(first);

        while self.peek().is_ascii_digit() {
            number.push(self.advance());
        }

        let is_float = self.peek() == '.'
            && self.peek_next().is_ascii_digit();

        if is_float {
            number.push(self.advance());

            while self.peek().is_ascii_digit() {
                number.push(self.advance());
            }
        }

        let kind = if is_float {
            TokenKind::Float(number.parse().map_err(|_| {
                KodeError::new(
                    "Lexer",
                    "invalid floating-point number",
                    self.line,
                    start,
                )
            })?)
        } else {
            TokenKind::Integer(number.parse().map_err(|_| {
                KodeError::new(
                    "Lexer",
                    "integer is outside the supported i64 range",
                    self.line,
                    start,
                )
            })?)
        };

        self.tokens.push(Token {
            kind,
            line: self.line,
            column: start,
        });

        Ok(())
    }

    fn scan_string(&mut self) -> Result<(), KodeError> {
        let start = self.column - 1;
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();

            if ch == '\n' {
                return Err(KodeError::new(
                    "Lexer",
                    "unterminated string; newline inside string",
                    self.line,
                    start,
                ));
            }

            if ch == '\\' {
                if self.is_at_end() {
                    break;
                }

                let escaped = self.advance();

                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),

                    other => {
                        return Err(self.error(&format!(
                            "unsupported escape sequence '\\{}'",
                            other
                        )));
                    }
                }
            } else {
                value.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(KodeError::new(
                "Lexer",
                "unterminated string",
                self.line,
                start,
            ));
        }

        self.advance();

        self.tokens.push(Token {
            kind: TokenKind::String(value),
            line: self.line,
            column: start,
        });

        Ok(())
    }

    fn add(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            line: self.line,
            column: self.column - 1,
        });
    }

    fn error(&self, message: &str) -> KodeError {
        KodeError::new(
            "Lexer",
            message,
            self.line,
            self.column.saturating_sub(1),
        )
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.current];
        self.current += 1;
        self.column += 1;
        ch
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> char {
        self.chars.get(self.current).copied().unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.chars.get(self.current + 1).copied().unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }
}