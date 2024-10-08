use super::Keywords;
use crate::lexical::AlphanumericLiteral;
use crate::lexical::Token;
use crate::lexical::TokenType;
use std::cell::RefCell;
use std::rc::Rc;

use crate::throw::{self, Error};

pub struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    throw: Rc<RefCell<Error>>,
    keywords: Keywords<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str, throw: Rc<RefCell<Error>>, keywords: Keywords<'a>) -> Self {
        Self {
            source: source.to_string(),
            throw: throw.clone(),
            keywords,
            ..Default::default()
        }
    }

    pub fn run(&mut self, is_printed: bool) -> &Vec<Token> {
        self.scan_tokens();
        if is_printed {
            self.print();
        }
        &self.tokens
    }

    fn print(&self) {
        println!("\n");
        for t in &self.tokens {
            println!("{:?}", t);
        }
    }

    // context
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source
                .chars()
                .nth(self.current)
                .expect("fn peek in Scanner.rs returned a char which is out of bounds.")
        }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source
                .chars()
                .nth(self.current + 1)
                .expect("fn peek_next in Scanner.rs returned a char which is out of bounds.")
        }
    }

    fn advance(&mut self) -> char {
        let value = self
            .source
            .chars()
            .nth(self.current)
            .expect("current scanner charecter out of bounds. check: is_at_end()");
        self.current += 1;
        value
    }

    // control flow
    fn match_token(&mut self, excepted: char) -> bool {
        if self.is_at_end() || self.source.chars().nth(self.current) != Some(excepted) {
            false
        } else {
            self.current += 1;
            true
        }
    }

    // literals
    //string
    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.throw
                .borrow_mut()
                .error(self.line, "Unterminated string.");
            return;
        }

        self.advance();
        let value: String = self
            .source
            .chars()
            .skip(self.start + 1)
            .take(self.current - self.start - 2)
            .collect();
        self.add_token(TokenType::String, Some(AlphanumericLiteral::String(value)));
    }

    //number
    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = &self.source[self.start..self.current];
        self.add_token(
            TokenType::Number,
            Some(
                value
                    .parse::<f64>()
                    .map(AlphanumericLiteral::Number)
                    .expect("Failed to parse number literal."),
            ),
        );
    }

    // token management
    fn add_token(&mut self, token_type: TokenType, literal: Option<AlphanumericLiteral>) {
        let text: &str = &self.source[self.start..self.current];

        match literal {
            Some(literal) => {
                self.tokens
                    .push(Token::new(token_type, Some(literal), text, self.line));
            }
            None => {
                self.tokens
                    .push(Token::new(token_type, None, text, self.line));
            }
        }
    }

    //reserved words & identifiers
    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text: &str = &self.source[self.start..self.current];
        let token_type = self.keywords.get(text);

        match token_type {
            Some(token_type) => {
                self.add_token(token_type.clone(), None);
            }
            None => {
                self.add_token(TokenType::Identifier, None);
            }
        }
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn scan_token(&mut self) {
        let c = &self.advance();

        match c {
            //single digit token
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '@' => self.add_token(TokenType::Annotation, None),
            '#' => self.add_token(TokenType::Document, None),
            ':' => self.add_token(TokenType::Colon, None),
            '?' => self.add_token(TokenType::Question, None),

            //double digit tokens
            '!' => {
                let token_type = if self.match_token('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type, None);
            }
            '=' => {
                let token_type = if self.match_token('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type, None);
            }
            '<' => {
                let token_type = if self.match_token('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type, None);
            }
            '>' => {
                let token_type = if self.match_token('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type, None);
            }
            '-' => {
                // check for comments
                let token_type = if self.match_token('>') {
                    TokenType::ReturnType
                } else {
                    TokenType::Minus
                };
                self.add_token(token_type, None);
            }
            '/' => {
                // check for comments
                if self.match_token('/') {
                    // Single-line comment
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_token('*') {
                    // Multi-line comment
                    while !self.is_at_end() {
                        if self.peek() == '*' && self.peek_next() == '/' {
                            self.advance(); // consume '*'
                            self.advance(); // consume '/'
                            break;
                        }
                        if self.peek() == '\n' {
                            self.line += 1;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }

            // string literal
            '"' => self.string(),

            // whitespace
            ' ' | '\r' | '\t' | '\n' => {
                // Ignore whitespace
                if *c == '\n' {
                    self.line += 1;
                }
            }
            _ => {
                if self.is_digit(*c) {
                    self.number()
                } else if self.is_alpha(*c) {
                    self.identifier();
                } else {
                    self.throw
                        .borrow_mut()
                        .error(self.line, "Unexpected character.");
                }
            }
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
    }
}

impl Default for Scanner<'_> {
    fn default() -> Self {
        Self {
            source: String::new(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            throw: Rc::new(RefCell::new(throw::Error::default())),
            keywords: Keywords::new(),
        }
    }
}
