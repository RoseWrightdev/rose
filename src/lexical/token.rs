use crate::lexical::TokenType;
use crate::lexical::Literal;
use std::fmt;

pub struct Token {
    token_type: TokenType,
    text: String,
    literal: Option<Literal>,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, literal: Option<Literal>, text: &str, line: usize) -> Self {
        Self {
            token_type,
            text: text.to_string(),
            literal,
            line,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nToken: {:?}\ntext: {:#?}\nliteral: {}\nline: {:?}\n",
            self.token_type,
            self.text,
            match &self.literal {
                Some(literal) => format!("{:?}", literal),
                None => "None".to_string(),
            },
            self.line
        )
    }
}

