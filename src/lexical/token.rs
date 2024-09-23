use crate::lexical::Literal;
use crate::lexical::TokenType;
use std::fmt;

pub struct Token {
    token_type: TokenType,
    text: String,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, _literal: Option<Literal>, text: &str, line: usize) -> Self {
        Self {
            token_type,
            text: text.to_string(),
            line,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token_type_str = format!("{:?}", self.token_type);
        let max_token_type_len = 14;
        write!(
            f,
            "({:?}) {:<width$} {}",
            self.line, token_type_str, self.text, width = max_token_type_len
        )?;
        Ok(())
    }
}
