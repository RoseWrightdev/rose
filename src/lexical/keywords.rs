use super::TokenType; // Assuming TokenType is in the parent module or correctly pathed
use std::collections::HashMap;

pub struct Keywords<'a> {
    hashmap: std::collections::HashMap<&'a str, TokenType>,
}

impl<'a> Keywords<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&TokenType> {
        self.hashmap.get(key)
    }
}

impl<'a> Default for Keywords<'a> {
    fn default() -> Self {
        let mut hashmap: HashMap<&str, TokenType> = HashMap::new();
        hashmap.insert("and", TokenType::And);          
        hashmap.insert("async", TokenType::Async);        
        hashmap.insert("await", TokenType::Await);        
        hashmap.insert("else", TokenType::Else);
        hashmap.insert("enum", TokenType::Enum);
        hashmap.insert("false", TokenType::False);
        hashmap.insert("fn", TokenType::Fn);            
        hashmap.insert("for", TokenType::For);
        hashmap.insert("if", TokenType::If);
        hashmap.insert("in", TokenType::In);
        hashmap.insert("let", TokenType::Let);          
        hashmap.insert("match", TokenType::Match);
        hashmap.insert("null", TokenType::Null);
        hashmap.insert("or", TokenType::Or);            
        hashmap.insert("return", TokenType::Return);
        hashmap.insert("struct", TokenType::Struct);
        hashmap.insert("true", TokenType::True);
        hashmap.insert("while", TokenType::While);

        Self { hashmap }
    }
}