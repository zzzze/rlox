use std::{fs::File, io::{Read, Write, BufRead, BufReader}};

pub struct Interpreter<'a> {
    input: &'a mut dyn BufRead,
    output: &'a mut dyn Write,
}

pub fn new <'a> (input: &'a mut impl BufRead, output: &'a mut impl Write) -> Interpreter<'a> {
    Interpreter { input, output }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ParameterError
}

impl <'a> Interpreter<'a> {
    pub fn exec(&mut self, args: Vec<String>) -> Result<(), Error> {
        if args.len() > 2 {
            self.output.write_all(b"Usage: rlox [script]\n").unwrap();
            return Err(Error::ParameterError);
        } else if args.len() == 2 {
            self.run_file(args[1].clone()).unwrap();
        } else {
            self.run_prompt();
        }
        Ok(())
    }

    fn run_file(&mut self, path: String) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        self.run(contents);
        Ok(())
    }

    fn run_prompt(&mut self) {
        loop {
            self.output.write_all(b"> ").unwrap();
            let mut line = String::new();
            self.input.read_line(&mut line).unwrap();
            if line == "exit" {
                break;
            }
            self.run(line);
        }
    }

    #[cfg(not(test))]
    fn run(&mut self, source: String) {
        println!("{}", source);
    }

    #[cfg(test)]
    fn run(&mut self, source: String) {
        self.output.write_all(source.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_args() {
        let mut input = "".as_bytes();
        let mut output = Vec::new();
        let mut interpreter = new(&mut input, &mut output);
        let err = interpreter.exec(vec![String::from("a"), String::from("b"), String::from("c")]).err();
        assert_eq!(err, Some(Error::ParameterError));
        assert_eq!(output, b"Usage: rlox [script]\n");
    }

    #[test]
    fn run_file() {
        let mut input = "".as_bytes();
        let mut output = Vec::new();
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        tmpfile.write_all(b"hello\nworld").unwrap();
        let file_path = tmpfile.path().to_str().unwrap().to_string();
        let mut interpreter = new(&mut input, &mut output);
        let err = interpreter.exec(vec![String::from("rlox"), file_path]).err();
        assert_eq!(err, None);
        assert_eq!(output, b"hello\nworld");
    }

    #[test]
    fn run_prompt() {
        let mut input = "hello\nworld\nexit".as_bytes();
        let mut output = Vec::new();
        let mut interpreter = new(&mut input, &mut output);
        let err = interpreter.exec(vec![]).err();
        assert_eq!(err, None);
        assert_eq!(output, b"> hello\n> world\n> ");
    }
}
