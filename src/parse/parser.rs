use crate::lexical::Token;

pub struct Parser {
    tokens: Vec<Token>,
    current: u32,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self{
        Self {
            tokens,
            ..Default::default()
        } 
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self { 
            tokens: Default::default(),
            current: 0 
        }
    }
}