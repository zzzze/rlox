use crate::token::{Token, TokenType, Literal};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(TokenType::EOF, "".to_string(), Literal::Nil, self.line));
        self.tokens.clone()
    }

    pub fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            _ => todo!()
        }
    }

    pub fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap_or_default()
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
