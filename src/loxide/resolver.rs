use std::collections::HashMap;

use thiserror::Error;

use super::{
    ast::{Expr, Stmt, Visitor},
    interpreter::Interpreter,
    token::Token,
};

#[derive(Debug, Error)]
pub enum Error {}

type Result<T = (), E = Error> = std::result::Result<T, E>;

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            scopes: Vec::new(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve(&mut self, statements: &[Stmt]) -> Result {
        for stmt in statements {
            self.visit_stmt(stmt)?;
        }
        Ok(())
    }

    fn declare(&mut self, name: &Token) -> Result {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.get_lexeme(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) -> Result {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.get_lexeme(), true);
        }
        Ok(())
    }
}

/*
   A block statement introduces a new scope for the statements it contains.
   A function declaration introduces a new scope for its body and binds its parameters in that scope.
   A variable declaration adds a new variable to the current scope.
   Variable and assignment expressions need to have their variables resolved.
*/

impl Visitor<Result, Result> for Resolver {
    fn visit_expr(&mut self, expr: &Expr) -> Result {
        todo!()
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result {
        match stmt {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve(statements)?;
                self.end_scope();
                Ok(())
            }

            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(initializer) = initializer {
                    self.visit_expr(initializer)?;
                }
                self.define(name)?;
                Ok(())
            }

            _ => todo!(),
        }
    }
}
