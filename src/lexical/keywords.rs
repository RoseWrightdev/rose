use std::collections::HashMap;

use super::TokenType;

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
        hashmap.insert("class", TokenType::Class);
        hashmap.insert("else", TokenType::Else);
        hashmap.insert("false", TokenType::False);
        hashmap.insert("for", TokenType::For);
        hashmap.insert("fun", TokenType::Fun);
        hashmap.insert("if", TokenType::If);
        hashmap.insert("nil", TokenType::Nil);
        hashmap.insert("or", TokenType::Or);
        hashmap.insert("print", TokenType::Print);
        hashmap.insert("return", TokenType::Return);
        hashmap.insert("super", TokenType::Super);
        hashmap.insert("this", TokenType::This);
        hashmap.insert("true", TokenType::True);
        hashmap.insert("var", TokenType::Var);
        hashmap.insert("while", TokenType::While);

        Self { hashmap }
    }
}
