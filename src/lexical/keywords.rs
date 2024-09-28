use super::TokenType;
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
        hashmap.insert("&&", TokenType::And);
        hashmap.insert("else", TokenType::Else);
        hashmap.insert("false", TokenType::False);
        hashmap.insert("for", TokenType::For);
        hashmap.insert("fn", TokenType::Function);
        hashmap.insert("if", TokenType::If);
        hashmap.insert("null", TokenType::Null);
        hashmap.insert("||", TokenType::Or);
        hashmap.insert("print", TokenType::Print);
        hashmap.insert("return", TokenType::Return);
        hashmap.insert("true", TokenType::True);
        hashmap.insert("let", TokenType::Var);
        hashmap.insert("while", TokenType::While);
        hashmap.insert("in", TokenType::In);
        hashmap.insert("enum", TokenType::Enum);
        hashmap.insert("struct", TokenType::Struct);
        hashmap.insert("match", TokenType::Match);

        Self { hashmap }
    }
}
