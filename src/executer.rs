use std::{io::Write, rc::Rc, sync::Mutex};
use super::error::{ErrorReporter, Error};

pub trait Executer {
    fn run(&mut self, source: String) -> Result<(), Error>;
}

pub struct EvalExecuter<'a> {
    had_error: bool,
    output: Rc<Mutex<&'a mut dyn Write>>,
}

impl<'a> EvalExecuter<'a> {
    pub fn new(output: Rc<Mutex<&'a mut dyn Write>>) -> Self {
        Self {
            had_error: false,
            output,
        }
    }
}

impl<'a> Executer for EvalExecuter<'a> {
    fn run(&mut self, source: String) -> Result<(), Error> {
        println!("{}", source);
        Ok(())
    }
}

impl<'a> ErrorReporter for EvalExecuter<'a> {
    fn report(&mut self, err: Error) -> Result<(), Error> {
        match err {
            Error::ParseError{..} => {
                self.output.lock().unwrap().write_all(format!("{}", err).as_bytes()).map_err(Error::IOError)?;
                self.had_error = true;
                Ok(())
            },
            _ => Err(err),
        }
    }
}
