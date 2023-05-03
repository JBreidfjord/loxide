use std::{collections::HashMap, fmt};

use crate::loxide::token::Token;

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
        Ok(Value::Instance(Instance::new(self.clone())))
    }
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

#[derive(Clone)]
pub struct Instance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        self.fields.get(&name.get_lexeme()).cloned()
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance of {}>", self.class.name)
    }
}
