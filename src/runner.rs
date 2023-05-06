use std::{io::Write, rc::Rc, sync::Mutex};
use super::error::{ErrorReporter, LoxError};

pub trait Runner {
    fn run(&mut self, source: String) -> Result<(), LoxError>;
}

pub struct LoxRunner<'a> {
    had_error: bool,
    output: Rc<Mutex<&'a mut dyn Write>>,
}

impl<'a> LoxRunner<'a> {
    pub fn new(output: Rc<Mutex<&'a mut dyn Write>>) -> Self {
        Self {
            had_error: false,
            output,
        }
    }
}

impl<'a> Runner for LoxRunner<'a> {
    fn run(&mut self, source: String) -> Result<(), LoxError> {
        let mut output = self.output.lock().unwrap();
        output.write_all(source.as_bytes())?;
        output.write_all(b"\n")?;
        output.flush()?;
        Ok(())
    }
}

impl<'a> ErrorReporter for LoxRunner<'a> {
    fn report(&mut self, err: LoxError) -> Result<(), LoxError> {
        match err {
            LoxError::ParseError{..} => {
                self.output.lock().unwrap().write_all(format!("{}", err).as_bytes())?;
                self.had_error = true;
                Ok(())
            },
            _ => Err(err),
        }
    }
}
