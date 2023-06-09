use crate::{token::{Token, TokenType, Literal}, error::LoxError};
use phf::phf_map;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "fun" => TokenType::Fun,
    "for" => TokenType::For,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
};

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token::new(TokenType::EOF, "", Literal::Nil, self.line));
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                if self.match_('=') {
                    self.add_token(TokenType::BangEqual, None)
                } else {
                    self.add_token(TokenType::Bang, None)
                }
            },
            '=' => {
                if self.match_('=') {
                    self.add_token(TokenType::EqualEqual, None)
                } else {
                    self.add_token(TokenType::Equal, None)
                }
            },
            '<' => {
                if self.match_('=') {
                    self.add_token(TokenType::LessEqual, None)
                } else {
                    self.add_token(TokenType::Less, None)
                }
            },
            '>' => {
                if self.match_('=') {
                    self.add_token(TokenType::GreaterEqual, None)
                } else {
                    self.add_token(TokenType::Greater, None)
                }
            },
            '/' => {
                if self.match_('/') {
                    self.comment()
                } else {
                    self.add_token(TokenType::Slash, None)
                }
            }
            ' ' | '\r' | '\t' => Ok(()),
            '\n' => Ok(self.line += 1),
            '"' => self.string(),
            c => {
                if self.is_digit(c) {
                    self.number()
                } else if self.is_alpha(c) {
                    self.identifier()
                } else {
                    Err(LoxError::ParseError {
                        line: self.line as usize,
                        where_: "".to_string(),
                        message: "Unexpected character.".to_string(),
                    })
                }
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal<'a>>) -> Result<(), LoxError> {
        let text = &self.source[self.start..self.current];
        let token = if let Some(literal) = literal {
            Token::new(token_type, text, literal, self.line)
        } else {
            Token::new(token_type, text, Literal::Nil, self.line)
        };
        Ok(self.tokens.push(token))
    }

    pub fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap_or_default()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn match_(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current) != Some(expected) {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0'
        }
        return self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0'
        }
        return self.source.chars().nth(self.current + 1).unwrap()
    }

    fn comment(&mut self) -> Result<(), LoxError> {
        Ok(while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        })
    }

    fn string(&mut self) -> Result<(), LoxError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(LoxError::ParseError {
                line: self.line as usize,
                where_: "".to_string(),
                message: "Unterminated string.".to_string(),
            })
        }
        // The closing ".
        self.advance();
        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(TokenType::String, Some(Literal::String(value)))
    }

    fn number(&mut self) -> Result<(), LoxError> {
        while self.is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        let value: f64 = self.source[self.start..self.current].parse().unwrap();
        self.add_token(TokenType::Number, Some(Literal::Number(value)))
    }

    fn identifier(&mut self) -> Result<(), LoxError> {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let text = self.source[self.start..self.current].to_string();
        if let Some(keyword) = KEYWORDS.get(&text) {
            return self.add_token(keyword.clone(), None);
        }
        self.add_token(TokenType::Identifier, None)
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn scan_tokens() {
        let source = "{,.-}\n(+*;)\n//Hello World!!\n\r \t/!====>>=<<=\"Hello\nWorld!!\"12.34or,hello_world;
            and class else false for fun if nil or print return super this true var while
        ";
        let expected = vec![
            Token::new(TokenType::LeftBrace, "{", Literal::Nil, 1),
            Token::new(TokenType::Comma, ",", Literal::Nil, 1),
            Token::new(TokenType::Dot, ".", Literal::Nil, 1),
            Token::new(TokenType::Minus, "-", Literal::Nil, 1),
            Token::new(TokenType::RightBrace, "}", Literal::Nil, 1),
            Token::new(TokenType::LeftParen, "(", Literal::Nil, 2),
            Token::new(TokenType::Plus, "+", Literal::Nil, 2),
            Token::new(TokenType::Star, "*", Literal::Nil, 2),
            Token::new(TokenType::Semicolon, ";", Literal::Nil, 2),
            Token::new(TokenType::RightParen, ")", Literal::Nil, 2),
            Token::new(TokenType::Slash, "/", Literal::Nil, 4),
            Token::new(TokenType::BangEqual, "!=", Literal::Nil, 4),
            Token::new(TokenType::EqualEqual, "==", Literal::Nil, 4),
            Token::new(TokenType::Equal, "=", Literal::Nil, 4),
            Token::new(TokenType::Greater, ">", Literal::Nil, 4),
            Token::new(TokenType::GreaterEqual, ">=", Literal::Nil, 4),
            Token::new(TokenType::Less, "<", Literal::Nil, 4),
            Token::new(TokenType::LessEqual, "<=", Literal::Nil, 4),
            Token::new(TokenType::String, "\"Hello\nWorld!!\"", Literal::String("Hello\nWorld!!"), 5),
            Token::new(TokenType::Number, "12.34", Literal::Number(12.34), 5),
            Token::new(TokenType::Or, "or", Literal::Nil, 5),
            Token::new(TokenType::Comma, ",", Literal::Nil, 5),
            Token::new(TokenType::Identifier, "hello_world", Literal::Nil, 5),
            Token::new(TokenType::Semicolon, ";", Literal::Nil, 5),
            // add, class, else, false, for, fun, if, nil, or, print, return, super, this, true, var, while
            Token::new(TokenType::And, "and", Literal::Nil, 6),
            Token::new(TokenType::Class, "class", Literal::Nil, 6),
            Token::new(TokenType::Else, "else", Literal::Nil, 6),
            Token::new(TokenType::False, "false", Literal::Nil, 6),
            Token::new(TokenType::For, "for", Literal::Nil, 6),
            Token::new(TokenType::Fun, "fun", Literal::Nil, 6),
            Token::new(TokenType::If, "if", Literal::Nil, 6),
            Token::new(TokenType::Nil, "nil", Literal::Nil, 6),
            Token::new(TokenType::Or, "or", Literal::Nil, 6),
            Token::new(TokenType::Print, "print", Literal::Nil, 6),
            Token::new(TokenType::Return, "return", Literal::Nil, 6),
            Token::new(TokenType::Super, "super", Literal::Nil, 6),
            Token::new(TokenType::This, "this", Literal::Nil, 6),
            Token::new(TokenType::True, "true", Literal::Nil, 6),
            Token::new(TokenType::Var, "var", Literal::Nil, 6),
            Token::new(TokenType::While, "while", Literal::Nil, 6),
            // eof
            Token::new(TokenType::EOF, "", Literal::Nil, 7),
        ];
        let mut scanner = Scanner::new(source);
        assert_eq!(scanner.scan_tokens().unwrap(), expected);
    }

    #[test]
    fn throw_unexpected_character() {
        let source = "；";
        let mut scanner = Scanner::new(source);
        match scanner.scan_tokens().unwrap_err() {
            LoxError::ParseError{line, message, ..} => {
                assert_eq!(line, 1);
                assert_eq!(message, "Unexpected character.");
            },
            _ => panic!("Invalid error"),
        }
    }

    #[test]
    fn throw_unterminated_string() {
        let source = "\"Hello World!!";
        let mut scanner = Scanner::new(source);
        match scanner.scan_tokens().unwrap_err() {
            LoxError::ParseError{line, message, ..} => {
                assert_eq!(line, 1);
                assert_eq!(message, "Unterminated string.");
            },
            _ => panic!("Invalid error"),
        }
    }
}
