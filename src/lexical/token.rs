use crate::lexical::LiteralValue; // UPDATED: Was AlphanumericLiteral
use crate::lexical::TokenType;
use std::fmt;

#[derive(Clone, PartialEq)] // Added PartialEq, Debug is implemented manually below
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,               // RENAMED: from text, made public
    pub literal: Option<LiteralValue>, // ADDED: to store the actual value of literals
    pub line: usize,                  // Made public
}

impl Token {
    // Updated signature to match scanner calls and use LiteralValue
    pub fn new(
        token_type: TokenType,
        literal: Option<LiteralValue>, // UPDATED: Type and name (was _literal: Option<AlphanumericLiteral>)
        lexeme: &str,                  // UPDATED: Parameter name (was text: &str)
        line: usize,
    ) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(), // Store the lexeme
            literal,                   // Store the literal value
            line,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token_type_str = format!("{:?}", self.token_type);
        // Adjust padding based on the longest TokenType variant name if desired, or keep fixed.
        let type_padding = 14; 
        
        write!(
            f,
            "Token {{ line: {:>3}, type: {:<type_padding$}, lexeme: \"{}\"",
            self.line,
            token_type_str,
            self.lexeme // Use self.lexeme
        )?;

        if let Some(literal) = &self.literal {
            write!(f, ", literal: {:?}", literal)?;
        }

        write!(f, " }}")?;
        Ok(())
    }
}