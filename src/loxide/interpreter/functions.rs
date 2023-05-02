use std::fmt;

use crate::loxide::{ast::Stmt, token::Token};

use super::{environment::Environment, value::Value, Error, Interpreter, Result};

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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FunctionDeclaration {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Clone)]
pub struct Function {
    pub declaration: FunctionDeclaration,
    pub closure: Environment,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        let mut environment = self.closure.nest();

        for (param, arg) in self.declaration.params.iter().zip(arguments) {
            environment.define(param.get_lexeme(), arg);
        }

        match interpreter.execute_block(&self.declaration.body, environment) {
            Err(Error::Return(value)) => Ok(value),
            Ok(_) => Ok(Value::Nil),
            Err(e) => Err(e),
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn `{}`>", self.declaration.name.get_lexeme())
    }
}
