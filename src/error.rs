use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid parameter")]
    InvalidParameter,
    #[error("[line {line}] Error {where_}: {message}")]
    ParseError {
        line: usize,
        where_: String,
        message: String,
    },
    #[error("Interpreter error")]
    InterpreterError,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub trait ErrorReporter {
    fn report(&mut self, err: Error) -> Result<(), Error>;
}
