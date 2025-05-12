use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TokenType {
    // --- Simple Punctuation ---
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Dot,          // .
    Semicolon,    // ;
    Colon,        // :
    Question,     // ?
    At,           // @
    Hash,         // #

    // --- Operators ---
    // Arithmetic
    Minus,        // -
    Plus,         // +
    Slash,        // /
    Star,         // *

    // Comparison & Assignment
    Bang,         // !
    BangEqual,    // !=
    Equal,        // =
    EqualEqual,   // ==
    Greater,      // >
    GreaterEqual, // >=
    Less,         // <
    LessEqual,    // <=

    // Arrows & Path Separators
    Arrow,        // ->
    FatArrow,     // =>
    DoubleColon,  // ::

    // --- Literals ---
    Identifier,   // my_var, functionName, TypeName
    String,       // Represents a string literal token, e.g., "hello"
    Number,       // Represents a number literal token (e.g., 123, 45.67); its specific value (int/float) is in LiteralValue
                  // HostPort was REMOVED here

    // --- Keywords ---
    And,          // and
    Async,        // async
    Await,        // await
    Else,         // else
    Enum,         // enum
    False,        // false (keyword for boolean literal)
    Fn,           // fn
    For,          // for
    If,           // if
    In,           // in
    Let,          // let
    Match,        // match
    Null,         // null (keyword for null literal)
    Or,           // or
    Return,       // return
    Struct,       // struct
    True,         // true (keyword for boolean literal)
    While,        // while

    // --- Control ---
    #[default]
    EndOfFile,    // EOF
}

// General LiteralValue enum (as you provided, which is up-to-date)
#[derive(Clone, PartialEq)]
pub enum LiteralValue {
    String(String),                             // For "hello"
    Integer(i64),                               // For 123
    Float(f64),                                 // For 3.14
    Boolean(bool),                              // For true, false
    Null,                                       // Represents the 'null' literal value
}

impl Debug for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::String(s) => write!(f, "\"{}\"", s),
            LiteralValue::Integer(n) => write!(f, "{}", n),
            LiteralValue::Float(n) => write!(f, "{}", n),
            LiteralValue::Boolean(b) => write!(f, "{}", b),
            LiteralValue::Null => write!(f, "null"),
        }
    }
}