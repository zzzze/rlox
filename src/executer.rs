use std::io::Write;
use super::error::{ErrorReporter, Error};

pub trait Executer {
    fn run(&mut self, source: String) -> Result<Vec<u8>, Error>;
}

pub struct EvalExecuter {
    had_error: bool,
    output_cache: Vec<u8>,
}

impl EvalExecuter {
    pub fn new() -> Self {
        Self {
            had_error: false,
            output_cache: Vec::new(),
        }
    }
}

impl Executer for EvalExecuter {
    fn run(&mut self, source: String) -> Result<Vec<u8>, Error> {
        println!("{}", source);
        Ok(self.output_cache.clone())
    }
}

impl ErrorReporter for EvalExecuter {
    fn report(&mut self, err: Error) -> Result<(), Error> {
        match err {
            Error::ParseError{..} => {
                self.output_cache.write_all(format!("{}", err).as_bytes()).map_err(Error::IOError)?;
                self.had_error = true;
                Ok(())
            },
            _ => Err(err),
        }
    }
}
