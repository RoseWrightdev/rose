use super::Keywords;
use crate::lexical::LiteralValue;
use crate::lexical::Token;
use crate::lexical::TokenType;
use std::cell::RefCell;
use std::rc::Rc;

use crate::throw::{self, Error};

pub struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,
    start: usize,    // Byte index of the start of the current lexeme
    current: usize,  // Current byte index being scanned
    line: usize,
    throw: Rc<RefCell<Error>>,
    keywords: Keywords<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str, throw: Rc<RefCell<Error>>, keywords: Keywords<'a>) -> Self {
        Self {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            throw: throw.clone(),
            keywords,
        }
    }

    pub fn run(&mut self) -> &Vec<Token> {
        self.scan_tokens();
        &self.tokens
    }

    pub fn print_tokens(&self) {
        println!("\n--- Tokens ---");
        for t in &self.tokens {
            println!("{:?}", t);
        }
        println!("--------------");
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn current_char_info(&self) -> Option<(char, usize)> {
        if self.is_at_end() {
            None
        } else {
            self.source[self.current..].chars().next().map(|c| (c, c.len_utf8()))
        }
    }

    fn peek(&self) -> char {
        self.current_char_info().map_or('\0', |(c, _)| c)
    }

    fn peek_next(&self) -> char {
        match self.current_char_info() {
            Some((_, current_char_len)) => {
                let next_char_start_idx = self.current + current_char_len;
                if next_char_start_idx >= self.source.len() {
                    '\0'
                } else {
                    self.source[next_char_start_idx..].chars().next().unwrap_or('\0')
                }
            }
            None => '\0', // CORRECTED from 'none'
        }
    }

    fn advance(&mut self) -> char {
        match self.current_char_info() {
            Some((c, len)) => {
                self.current += len;
                c
            }
            None => '\0', // CORRECTED from 'none'; Should be guarded by is_at_end() before calling
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        match self.current_char_info() {
            Some((c, len)) if c == expected => {
                self.current += len;
                true
            }
            _ => false,
        }
    }

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

        self.advance(); // Consume the closing "

        let opening_quote_len = '"'.len_utf8();
        let closing_quote_len = '"'.len_utf8();
        
        let value_start_idx = self.start + opening_quote_len;
        let value_end_idx = self.current - closing_quote_len;

        if value_start_idx <= value_end_idx {
            let value_str = &self.source[value_start_idx..value_end_idx];
            self.add_token(TokenType::String, Some(LiteralValue::String(value_str.to_string())));
        } else { // Handles the case of an empty string "" correctly
            self.add_token(TokenType::String, Some(LiteralValue::String("".to_string())));
        }
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        let mut is_float = false;
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            is_float = true;
            self.advance(); 
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let number_lexeme = &self.source[self.start..self.current];

        if is_float {
            match number_lexeme.parse::<f64>() {
                Ok(val) => self.add_token(TokenType::Number, Some(LiteralValue::Float(val))),
                Err(_) => self.throw.borrow_mut().error(
                    self.line,
                    &format!("Invalid floating-point number: '{}'", number_lexeme),
                ),
            }
        } else {
            match number_lexeme.parse::<i64>() {
                Ok(val) => self.add_token(TokenType::Number, Some(LiteralValue::Integer(val))),
                Err(_) => self.throw.borrow_mut().error(
                    self.line,
                    &format!("Invalid integer number: '{}'", number_lexeme),
                ),
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text_lexeme = &self.source[self.start..self.current];
        // Assuming Token::new expects: (TokenType, Option<LiteralValue>, &str (lexeme), usize (line))
        // This matches the corrected signature needed in token.rs
        self.tokens.push(Token::new(token_type, literal, text_lexeme, self.line));
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text_lexeme = &self.source[self.start..self.current];
        if let Some(keyword_type) = self.keywords.get(text_lexeme).cloned() {
            match keyword_type {
                TokenType::True => self.add_token(TokenType::True, Some(LiteralValue::Boolean(true))),
                TokenType::False => self.add_token(TokenType::False, Some(LiteralValue::Boolean(false))),
                TokenType::Null => self.add_token(TokenType::Null, Some(LiteralValue::Null)),
                _ => self.add_token(keyword_type, None),
            }
        } else {
            self.add_token(TokenType::Identifier, None);
        }
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '@' => self.add_token(TokenType::At, None),
            '#' => self.add_token(TokenType::Hash, None),
            '?' => self.add_token(TokenType::Question, None),

            ':' => {
                if self.match_char(':') {
                    self.add_token(TokenType::DoubleColon, None);
                } else {
                    self.add_token(TokenType::Colon, None);
                }
            }
            '-' => {
                if self.match_char('>') {
                    self.add_token(TokenType::Arrow, None);
                } else {
                    self.add_token(TokenType::Minus, None);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual, None);
                } else {
                    self.add_token(TokenType::Bang, None);
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual, None);
                } else if self.match_char('>') {
                    self.add_token(TokenType::FatArrow, None);
                } else {
                    self.add_token(TokenType::Equal, None);
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual, None);
                } else {
                    self.add_token(TokenType::Less, None);
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual, None);
                } else {
                    self.add_token(TokenType::Greater, None);
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    loop {
                        if self.is_at_end() {
                            self.throw.borrow_mut().error(self.line, "Unterminated block comment.");
                            break;
                        }
                        if self.peek() == '*' && self.peek_next() == '/' {
                            self.advance(); 
                            self.advance(); 
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
            '"' => self.string(),
            ' ' | '\r' | '\t' => { /* Ignore whitespace */ }
            '\n' => {
                self.line += 1;
            }
            _ => { 
                if c == '\0' && self.is_at_end() {
                    // EOF effectively handled by the main loop condition
                } else if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    self.throw.borrow_mut().error(self.line, &format!("Unexpected character: '{}'", c));
                }
            }
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current; 
            self.scan_token();
        }

        // Pass "" as the lexeme for EOF, and None for the literal value
        self.tokens.push(Token::new(
            TokenType::EndOfFile,
            None, // No literal value for EOF
            "",   // Empty lexeme for EOF
            self.line,
        ));
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