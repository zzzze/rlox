use rlox::{
    ast_printer::AstPrinter,
    error, runner,
    expr::{Binary, Expr, Grouping, Literal, Unary},
    lox,
    token::{Literal as LiteralToken, Token, TokenType},
};
use std::{
    io::{BufRead, Write},
    rc::Rc,
    sync::Mutex,
};

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let input: Rc<Mutex<&mut dyn BufRead>> = Rc::new(Mutex::new(&mut stdin));
    let output: Rc<Mutex<&mut dyn Write>> = Rc::new(Mutex::new(&mut stdout));
    let expression: Expr = Expr::Binary(Binary::new(
        Expr::Unary(Unary::new(
            Token::new(TokenType::Minus, "-", LiteralToken::Nil, 1),
            Expr::Literal(Literal::new(LiteralToken::Number(123f64))),
        )),
        Token::new(TokenType::Star, "*", LiteralToken::Nil, 1),
        Expr::Grouping(Grouping::new(Expr::Literal(Literal::new(
            LiteralToken::Number(45.67f64),
        )))),
    ));
    let mut printer: AstPrinter = AstPrinter;
    let result = printer.print(&expression) + "\n";
    std::io::stdout()
        .lock()
        .write_all(result.as_bytes())
        .unwrap();
    if let Err(err) = lox::new(
        input,
        output.clone(),
        runner::LoxRunner::new(output.clone()),
    )
    .exec(std::env::args().collect())
    {
        match err {
            error::LoxError::InvalidParameter => {
                std::process::exit(64);
            }
            error::LoxError::RuntimeError => {
                std::process::exit(65);
            }
            _ => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    }
}
