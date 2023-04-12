use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    InvalidParameter,
    IOError(#[from] std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidParameter => write!(f, "Invalid parameter"),
            Error::IOError(err) => write!(f, "IO error: {}", err),
        }
    }
}
