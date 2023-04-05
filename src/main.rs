mod interpreter;

fn main() {
    if let Err(err) = interpreter::new(&mut std::io::stdin().lock(), &mut std::io::stdout()).exec(std::env::args().collect()) {
        match err {
            interpreter::Error::ParameterError => {
                std::process::exit(64);
            },
        }
    }
}
