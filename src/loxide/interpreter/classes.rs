use std::fmt;

use super::{functions::Callable, value::Value, Interpreter, Result};

#[derive(Clone)]
pub struct Class {
    pub name: String,
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &mut Interpreter, _: Vec<Value>) -> Result<Value> {
        Ok(Value::Instance(Instance {
            class: self.clone(),
        }))
    }
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

#[derive(Clone)]
pub struct Instance {
    pub class: Class,
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance of {}>", self.class.name)
    }
}
