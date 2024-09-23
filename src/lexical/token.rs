use crate::lexical::TokenType;
use std::fmt;

pub struct Token {
    token_type: TokenType,
    text: String,
    literal: Option<String>,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, literal: Option<&str>, text: &str, line: &usize) -> Self {
        match literal {
            Some(literal) => Self {
                token_type,
                text: text.to_string(),
                literal: Some(literal.to_string()),
                line: *line,
            },
            None => Self {
                token_type,
                text: text.to_string(),
                literal: None,
                line: *line,
            },
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nToken: {:?}
            text: {:#?}
            literal: {:?}
            line: {:?}\n",
            self.token_type, self.text, self.literal, self.line
        )
    }
}
