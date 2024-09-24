use std::fmt::Debug;

#[derive(Debug, Clone)]
#[derive(Default)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    ReturnType,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    #[default]
    EndOfFile,
}


pub enum AlphanumericLiteral {
    String(String),
    Number(f64),
}

impl Debug for AlphanumericLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlphanumericLiteral::String(s) => write!(f, "{}", s),
            AlphanumericLiteral::Number(n) => write!(f, "{}", n),
        }
    }
}
