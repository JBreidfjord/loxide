use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::interpreter::Value;

// Cactus stack / parent-pointer tree
// Based on https://stackoverflow.com/a/48298865
pub struct Environment(Option<Rc<Scope>>);

struct Scope {
    variables: RefCell<HashMap<String, Value>>,
    enclosing: Environment,
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Environment {
    /// Create a new global environment scope.
    pub fn global() -> Self {
        Self(None)
    }

    /// Create a new nested environment scope.
    pub fn nest(&self) -> Self {
        let scope = Scope {
            variables: RefCell::new(HashMap::new()),
            enclosing: self.clone(),
        };
        Self(Some(Rc::new(scope)))
    }

    fn enclosing(&self) -> Self {
        self.0.as_ref().map_or(Self(None), |s| s.enclosing.clone())
    }

    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.0.as_ref() {
            scope.variables.borrow_mut().insert(name, value);
        }
    }

    pub fn lookup(&self, name: String) -> Option<Value> {
        // If the variable is not found in the current environment,
        // we recursively search the enclosing environment.
        if let Some(value) = self
            .0
            .as_ref()
            .and_then(|s| s.variables.borrow().get(&name).cloned())
        {
            Some(value)
        } else {
            self.enclosing().lookup(name)
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> bool {
        if let Some(scope) = self.0.as_ref() {
            if scope.variables.borrow().contains_key(&name) {
                scope.variables.borrow_mut().insert(name, value);
                return true;
            }

            // If the variable is not found in the current environment,
            // we recursively search the enclosing environment.
            return self.enclosing().assign(name, value);
        }
        false
    }
}
