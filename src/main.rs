use rlox::{ interpreter, executer, error };

fn main() {
    if let Err(err) = interpreter::new(
        &mut std::io::stdin().lock(),
        &mut std::io::stdout(),
        executer::EvalExecuter::new(),
    ).exec(std::env::args().collect()) {
        match err {
            error::Error::InvalidParameter => {
                std::process::exit(64);
            },
            error::Error::InterpreterError => {
                std::process::exit(65);
            },
            _ => {
                eprintln!("{}", err);
                std::process::exit(1);
            },
        }
    }
}
