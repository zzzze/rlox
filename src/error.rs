use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid parameter")]
    InvalidParameter,
    #[error("[line {line}] Error {where_}: {message}")]
    InterpreterError {
        line: usize,
        where_: String,
        message: String,
    },
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
