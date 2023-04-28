use std::{rc::Rc, sync::Mutex, io::{Write, BufRead}};

use rlox::{ lox, executer, error };

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let input: Rc<Mutex<&mut dyn BufRead>> = Rc::new(Mutex::new(&mut stdin));
    let output: Rc<Mutex<&mut dyn Write>> = Rc::new(Mutex::new(&mut stdout));
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
