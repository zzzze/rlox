use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("Invalid parameter")]
    InvalidParameter,
    #[error("[line {line}] Error {where_}: {message}")]
    ParseError {
        line: usize,
        where_: String,
        message: String,
    },
    #[error("Runtime error")]
    RuntimeError,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub trait ErrorReporter {
    fn report(&mut self, err: LoxError) -> Result<(), LoxError>;
}
