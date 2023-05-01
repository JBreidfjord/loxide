use std::fmt;

use super::{value::Value, Interpreter, Result};

type Function = fn(&mut Interpreter, Vec<Value>) -> Result<Value>;

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>;
    fn arity(&self) -> usize;
}

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    arity: usize,
    function: Function,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        (self.function)(interpreter, arguments)
    }
}

impl NativeFunction {
    pub fn new(name: String, arity: usize, function: Function) -> Self {
        Self {
            name,
            arity,
            function,
        }
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn `{}`>", self.name)
    }
}
