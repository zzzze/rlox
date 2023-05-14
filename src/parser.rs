use std::cell::RefCell;
use crate::{
    error::LoxError,
    expr::{Binary, Expr, Grouping, Literal, Unary},
    token::{Literal as LoxLiteral, Token, TokenType},
};

pub struct Parser<'a> {
    tokens: &'a Vec<Token<'a>>,
    current: RefCell<usize>,
}

fn error(token: Token, message: String) -> LoxError {
    if token.token_type == TokenType::EOF {
        LoxError::ParseError { line: token.line as usize, where_: String::from(" at end"), message }
    } else {
        LoxError::ParseError { line: token.line as usize, where_: format!(" at '{}'", token.lexeme), message }
    }
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token<'a>>) -> Self {
        Self { tokens, current: RefCell::new(0) }
    }

    fn parse(&self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => None,
        }
    }

    fn expression(&self) -> Result<Expr, LoxError> {
        return self.equality();
    }

    fn equality(&self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;
        while self.match_(&[&TokenType::BangEqual, &TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparison(&self) -> Result<Expr, LoxError> {
        let mut expr = self.term()?;
        while self.match_(&[
            &TokenType::Greater,
            &TokenType::GreaterEqual,
            &TokenType::Less,
            &TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn term(&self) -> Result<Expr, LoxError> {
        let mut expr = self.factor()?;
        while self.match_(&[&TokenType::Plus, &TokenType::Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn factor(&self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;
        while self.match_(&[&TokenType::Star, &TokenType::Slash]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&self) -> Result<Expr, LoxError> {
        while self.match_(&[&TokenType::Bang, &TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary::new(operator, right)));
        }
        self.primary()
    }

    fn primary(&self) -> Result<Expr, LoxError> {
        if self.match_(&[&TokenType::False]) {
            return Ok(Expr::Literal(Literal::new(LoxLiteral::Boolean(false))));
        }
        if self.match_(&[&TokenType::True]) {
            return Ok(Expr::Literal(Literal::new(LoxLiteral::Boolean(true))));
        }
        if self.match_(&[&TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::new(LoxLiteral::Nil)));
        }
        if self.match_(&[&TokenType::Number, &TokenType::String]) {
            return Ok(Expr::Literal(Literal::new(self.previous().literal)));
        }
        if self.match_(&[&TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Grouping::new(expr)));
        }
        Err(error(self.peek(), String::from("Expect expression.")))
    }

    fn match_(&self, types: &[&TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == *token_type
        }
    }

    fn advance(&self) -> Token<'a> {
        if !self.is_at_end() {
            let mut current = self.current.borrow_mut();
            *current += 1;
        }
        return self.previous();
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token<'a> {
        println!("{:?}", self.current);
        self.tokens[self.current.borrow().to_owned()].clone()
    }

    fn previous(&self) -> Token<'a> {
        self.tokens[self.current.borrow().to_owned() - 1].clone()
    }

    fn consume(&self, token_type: TokenType, message: &'a str) -> Result<Token<'a>, LoxError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(error(self.peek(), message.to_string()))
        }
    }

    fn synchronize(&self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class |
                TokenType::Fun |
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType:: While |
                TokenType::Print |
                TokenType::Return => return,
                _ => self.advance(),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{token, ast_printer::AstPrinter};
    use super::*;

    #[test]
    fn parse_expr() {
        let tokens = vec![
            token::Token::new(TokenType::Minus, "-", LoxLiteral::Nil, 1),
            token::Token::new(TokenType::Number, "123", LoxLiteral::Number(123.0), 1),
            token::Token::new(TokenType::Star, "*", LoxLiteral::Nil, 1),
            token::Token::new(TokenType::LeftParen, "(", LoxLiteral::Nil, 1),
            token::Token::new(TokenType::Number, "45.67", LoxLiteral::Number(45.67), 1),
            token::Token::new(TokenType::RightParen, ")", LoxLiteral::Nil, 1),
            token::Token::new(TokenType::EOF, "", LoxLiteral::Nil, 1),
        ];
        let parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let mut printer: AstPrinter = AstPrinter;
        let result = printer.print(&expr);
        assert_eq!(result, "(* (- 123) (group 45.67))");
    }
}
