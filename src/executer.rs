use super::error::Error;

pub trait Executer {
    fn run(&mut self, source: String) -> Result<(), Error>;
}

pub struct EvalExecuter;

impl Executer for EvalExecuter {
    fn run(&mut self, source: String) -> Result<(), Error> {
        println!("{}", source);
        Ok(())
    }
}
