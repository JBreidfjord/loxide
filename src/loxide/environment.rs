use std::collections::HashMap;

use super::interpreter::Value;

pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: String) -> Option<Value> {
        self.variables.get(&name).cloned()
    }

    pub fn assign(&mut self, name: String, value: Value) -> bool {
        if !self.variables.contains_key(&name) {
            return false;
        }

        self.variables.insert(name, value);
        true
    }
}
