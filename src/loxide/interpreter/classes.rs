use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::loxide::token::Token;

use super::{functions::Callable, value::Value, Interpreter, Result};

#[derive(Clone)]
pub struct Class {
    pub name: String,
    pub superclass: Option<Box<Value>>,
    pub methods: HashMap<String, Value>,
}

impl Class {
    pub fn find_method(&self, name: &str) -> Option<Value> {
        self.methods.get(name).cloned()
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        // If the class has an init method, return its arity
        if let Some(init) = self.find_method("init") {
            match init {
                Value::Function(func) => func.arity(),
                _ => unreachable!("Expected function for init method"),
            }
        } else {
            0
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        let instance = Instance::new(self.clone());
        // Bind and call the init method if it exists
        if let Some(init) = self.find_method("init") {
            match init {
                Value::Function(func) => func.bind(instance.clone()).call(interpreter, arguments),
                _ => unreachable!("Expected function for init method"),
            }?;
        }

        Ok(Value::Instance(instance))
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
    fields: Rc<RefCell<HashMap<String, Value>>>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        if let Some(value) = self.fields.borrow().get(&name.get_lexeme()) {
            Some(value.clone())
        } else {
            self.class
                .find_method(&name.get_lexeme())
                .map(|method| match method {
                    Value::Function(func) => Value::Function(func.bind(self.clone())),
                    _ => method,
                })
        }
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.borrow_mut().insert(name.get_lexeme(), value);
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance of {}>", self.class.name)
    }
}
