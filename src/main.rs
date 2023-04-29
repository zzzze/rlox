use std::{rc::Rc, sync::Mutex, io::{Write, BufRead}};
use rlox::{ lox, executer, error, expr, token, ast_printer::AstPrinter };

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let input: Rc<Mutex<&mut dyn BufRead>> = Rc::new(Mutex::new(&mut stdin));
    let output: Rc<Mutex<&mut dyn Write>> = Rc::new(Mutex::new(&mut stdout));
    let expression: expr::Expr = expr::Expr::Binary(expr::Binary::new(
        Box::new(expr::Expr::Unary(expr::Unary::new(
            token::Token::new(token::TokenType::Minus, "-".to_string(), token::Literal::Nil, 1),
            Box::new(expr::Expr::Literal(expr::Literal::new(token::Literal::Number(123f64)))),
        ))),
        token::Token::new(token::TokenType::Star, "*".to_string(), token::Literal::Nil, 1),
        Box::new(expr::Expr::Grouping(expr::Grouping::new(
            Box::new(expr::Expr::Literal(expr::Literal::new(token::Literal::Number(45.67f64)))),
        ))),
    ));
    let mut printer: AstPrinter = AstPrinter;
    let result = printer.print(&expression) + "\n";
    std::io::stdout().lock().write_all(result.as_bytes()).unwrap();
    if let Err(err) = lox::new(
        input,
        output.clone(),
        executer::EvalExecuter::new(output.clone()),
    ).exec(std::env::args().collect()) {
        match err {
            error::LoxError::InvalidParameter => {
                std::process::exit(64);
            },
            error::LoxError::RuntimeError => {
                std::process::exit(65);
            },
            _ => {
                eprintln!("{}", err);
                std::process::exit(1);
            },
        }
    }
}
