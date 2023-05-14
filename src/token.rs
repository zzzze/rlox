use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EOF
}

pub trait Object: std::fmt::Debug {
    fn box_clone(&self) -> Box<dyn Object>;
}

impl Clone for Box<dyn Object> {
    fn clone(&self) -> Box<dyn Object> {
        self.box_clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'a> {
    String(&'a str),
    Boolean(bool),
    Number(f64),
    Nil,
}

impl<'a> std::fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::String(literal) => write!(f, "{}", literal),
            Self::Boolean(literal) => write!(f, "{}", literal),
            Self::Number(literal) => write!(f, "{}", literal),
        }
    }
}

pub enum Value<'a> {
    Literal(Literal<'a>),
    Object(Box<dyn Object>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub literal: Literal<'a>,
    pub line: u32,
}

impl<'a> Token<'a> {
    pub fn new(token_type: TokenType, lexeme: &'a str, literal: Literal<'a>, line: u32) -> Token<'a> {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}
