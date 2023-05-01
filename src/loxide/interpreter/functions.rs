use std::fmt;

use crate::loxide::{ast::Stmt, token::Token};

use super::{value::Value, Interpreter, Result};

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>;
    fn arity(&self) -> usize;
}

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub function: fn(&mut Interpreter, Vec<Value>) -> Result<Value>,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        (self.function)(interpreter, arguments)
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn `{}`>", self.name)
    }
}

#[derive(Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        let mut environment = interpreter.globals.nest();

        for (param, arg) in self.params.iter().zip(arguments) {
            environment.define(param.get_lexeme(), arg);
        }

        interpreter.execute_block(&self.body, environment)?;

        Ok(Value::Nil)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn `{}`>", self.name.get_lexeme())
    }
}
