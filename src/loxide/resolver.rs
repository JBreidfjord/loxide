use std::collections::HashMap;

use thiserror::Error;

use super::{
    ast::{Expr, Stmt, Visitor},
    interpreter::{functions::FunctionDeclaration, Interpreter},
    token::Token,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't read local variable in its own initializer.")]
    SelfReferencedInitializer,
}

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

    fn resolve_local(&mut self, expr: &Expr, name: &Token) -> Result {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.get_lexeme()) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                return Ok(());
            }
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

    fn resolve_function(&mut self, declaration: &FunctionDeclaration) -> Result {
        self.begin_scope();
        for param in &declaration.params {
            self.declare(param)?;
            self.define(param)?;
        }
        self.resolve(&declaration.body)?;
        self.end_scope();
        Ok(())
    }
}

impl Visitor<Result, Result> for Resolver {
    fn visit_expr(&mut self, expr: &Expr) -> Result {
        match expr {
            Expr::Variable(name) => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(false) = scope.get(&name.get_lexeme()) {
                        return Err(Error::SelfReferencedInitializer);
                    }
                }
                self.resolve_local(expr, name)
            }

            Expr::Assign { name, value } => {
                self.visit_expr(value)?;
                self.resolve_local(expr, name)
            }

            Expr::Binary { left, right, .. } => {
                self.visit_expr(left)?;
                self.visit_expr(right)
            }

            Expr::Call {
                callee, arguments, ..
            } => {
                self.visit_expr(callee)?;
                arguments.iter().try_for_each(|arg| self.visit_expr(arg))
            }

            Expr::Grouping(expr) => self.visit_expr(expr),

            Expr::Literal(_) => Ok(()),

            Expr::Logical { left, right, .. } => {
                self.visit_expr(left)?;
                self.visit_expr(right)
            }

            Expr::Unary { right, .. } => self.visit_expr(right),

            Expr::Lambda(declaration) => self.resolve_function(declaration),
        }
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

            Stmt::Function(declaration) => {
                self.declare(&declaration.name)?;
                self.define(&declaration.name)?;
                self.resolve_function(declaration)
            }

            Stmt::Expression(expr) => self.visit_expr(expr),

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit_expr(condition)?;
                self.visit_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.visit_stmt(else_branch)?;
                }
                Ok(())
            }

            Stmt::Print(expr) => self.visit_expr(expr),

            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.visit_expr(value)?;
                }
                Ok(())
            }

            Stmt::While { condition, body } => {
                self.visit_expr(condition)?;
                self.visit_stmt(body)
            }

            Stmt::Break => Ok(()),
        }
    }
}
