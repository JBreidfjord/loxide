use std::fmt;

use crate::loxide::{ast::Stmt, token::Token};

use super::{
    classes::Instance, environment::Environment, value::Value, Error, Interpreter, Result,
};

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
    pub is_init: bool,
}

impl Function {
    pub fn new(declaration: FunctionDeclaration, closure: Environment) -> Self {
        Self {
            declaration,
            closure,
            is_init: false,
        }
    }

    pub fn new_init(declaration: FunctionDeclaration, closure: Environment) -> Self {
        Self {
            declaration,
            closure,
            is_init: true,
        }
    }

    pub fn bind(self, instance: Instance) -> Self {
        let mut environment = self.closure.nest();
        environment.define("this".to_string(), Value::Instance(instance));
        Self {
            closure: environment,
            ..self
        }
    }
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

        let result = interpreter.execute_block(&self.declaration.body, environment);
        if self.is_init {
            // If this is an initializer, always return `this`
            Ok(self
                .closure
                .lookup_at(0, "this".to_string())
                .expect("Expected `this` to be defined in initializer"))
        } else {
            // Otherwise, return the result of the block
            match result {
                Err(Error::Return(value)) => Ok(value),
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(e),
            }
        }
    }
}

impl TryFrom<Value> for Function {
    type Error = Error;

    fn try_from(value: Value) -> Result<Function, Error> {
        match value {
            Value::Function(func) => Ok(func),
            _ => Err(Error::ConversionError {
                from: value,
                to: "<fn>".to_string(),
            }),
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn `{}`>", self.declaration.name.get_lexeme())
    }
}
