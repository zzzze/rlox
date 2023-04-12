mod interpreter;
mod executer;
mod error;

fn main() {
    if let Err(err) = interpreter::new(
        &mut std::io::stdin().lock(),
        &mut std::io::stdout(),
        executer::EvalExecuter
    ).exec(std::env::args().collect()) {
        match err {
            error::Error::InvalidParameter => {
                std::process::exit(64);
            },
            _ => {
                eprintln!("{}", err);
                std::process::exit(1);
            },
        }
    }
}
