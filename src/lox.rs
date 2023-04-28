use std::{
    fs::File,
    io::{
        Read, Write, BufRead, BufReader
    },
    rc::Rc,
    sync::Mutex,
};

use super::error::LoxError;
use super::executer::Executer;

pub struct Lox<'a, T: Executer> {
    input: Rc<Mutex<&'a mut dyn BufRead>>,
    output: Rc<Mutex<&'a mut dyn Write>>,
    executer: T,
}

pub fn new <'a, T: Executer> (input: Rc<Mutex<&'a mut dyn BufRead>>, output: Rc<Mutex<&'a mut dyn Write>>, executer: T) -> Lox<'a, T> {
    Lox { input, output, executer }
}

impl <'a, T: Executer> Lox<'a, T> {
    pub fn exec(&mut self, args: Vec<String>) -> Result<(), LoxError> {
        if args.len() > 2 {
            self.output.lock().unwrap().write_all(b"Usage: lox [script]\n").unwrap(); // FIXME
            return Err(LoxError::InvalidParameter);
        } else if args.len() == 2 {
            self.run_file(args[1].clone()).unwrap();
        } else {
            self.run_prompt()?;
        }
        Ok(())
    }

    fn run_file(&mut self, path: String) -> Result<(), LoxError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        self.run(contents, false)?;
        Ok(())
    }

    fn run_prompt(&mut self) -> Result<(), LoxError> {
        loop {
            self.output.lock().unwrap().write_all(b"> ")?;
            self.output.lock().unwrap().flush()?;
            let mut line = String::new();
            self.input.lock().unwrap().read_line(&mut line)?;
            line = line.trim_end().to_string();
            if line == String::from("exit") {
                break;
            }
            self.run(line, true)?;
        }
        Ok(())
    }

    fn run(&mut self, source: String, ignore_interpreter_error: bool) -> Result<(), LoxError> {
        if let Err(err) = self.executer.run(source) {
            match err {
                LoxError::RuntimeError => {
                    self.output.lock().unwrap().write_all(format!("{}", err).as_bytes())?;
                    if !ignore_interpreter_error {
                        return Err(err)
                    } else {
                        return Ok(())
                    }
                },
                _ => return Err(err),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExecuter<'a>(Rc<Mutex<&'a mut dyn Write>>);

    impl<'a> Executer for MockExecuter<'a> {
        fn run(&mut self, source: String) -> Result<(), LoxError> {
            self.0.lock().unwrap().write_all(source.as_bytes())?;
            Ok(())
        }
    }

    #[test]
    fn invalid_args() {
        let mut input = "".as_bytes();
        let mut output_buffer = Vec::new();
        let output: Rc<Mutex<&mut dyn Write>> = Rc::new(Mutex::new(&mut output_buffer));
        let mut interpreter = new(Rc::new(Mutex::new(&mut input)), output.clone(), MockExecuter(output.clone()));
        let err = interpreter.exec(vec![String::from("a"), String::from("b"), String::from("c")]).err();
        match err {
            Some(LoxError::InvalidParameter) => (),
            _ => panic!("Invalid error"),
        }
        assert_eq!(output_buffer, b"Usage: lox [script]\n");
    }

    #[test]
    fn run_file() {
        let mut input = "".as_bytes();
        let mut output_buffer = Vec::new();
        let output: Rc<Mutex<&mut dyn Write>> = Rc::new(Mutex::new(&mut output_buffer));
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        tmpfile.write_all(b"hello\nworld").unwrap();
        let file_path = tmpfile.path().to_str().unwrap().to_string();
        let mut interpreter = new(Rc::new(Mutex::new(&mut input)), output.clone(), MockExecuter(output.clone()));
        let err = interpreter.exec(vec![String::from("lox"), file_path]).err();
        assert_eq!(err.is_none(), true);
        assert_eq!(output_buffer, b"hello\nworld");
    }

    #[test]
    fn run_prompt() {
        let mut input = "hello\nworld\nexit".as_bytes();
        let mut output_buffer = Vec::new();
        let output: Rc<Mutex<&mut dyn Write>> = Rc::new(Mutex::new(&mut output_buffer));
        let mut interpreter = new(Rc::new(Mutex::new(&mut input)), output.clone(), MockExecuter(output.clone()));
        let err = interpreter.exec(vec![]).err();
        assert_eq!(err.is_none(), true);
        assert_eq!(output_buffer, b"> hello> world> ");
    }
}
