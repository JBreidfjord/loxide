use std::collections::HashMap;

use thiserror::Error;

use super::{
    ast::{Expr, Stmt, Visitor},
    interpreter::functions::FunctionDeclaration,
    token::Token,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't read local variable in its own initializer.")]
    SelfReferencedInitializer,

    #[error("A variable with name `{name}` was already declared in this scope.")]
    VariableAlreadyDeclared { name: String },

    #[error("Can't return from top-level code.")]
    ReturnOutsideFunction,

    #[error("Can't use `this` outside of a class.")]
    ThisOutsideClass,

    #[error("Internal error: {0}")]
    Internal(String),
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(PartialEq, Copy, Clone)]
enum FnType {
    None,
    Function,
    Method,
}

#[derive(PartialEq, Copy, Clone)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    locals: HashMap<Expr, usize>,
    current_fn: FnType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            locals: HashMap::new(),
            current_fn: FnType::None,
            current_class: ClassType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn run(mut self, statements: &[Stmt]) -> Result<HashMap<Expr, usize>, Vec<Error>> {
        let mut errors = Vec::new();
        for stmt in statements {
            match self.visit_stmt(stmt) {
                Ok(_) => (),
                Err(err) => errors.push(err),
            }
        }

        if errors.is_empty() {
            Ok(self.locals)
        } else {
            Err(errors)
        }
    }

    pub fn resolve(&mut self, statements: &[Stmt]) -> Result {
        statements.iter().try_for_each(|stmt| self.visit_stmt(stmt))
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.get_lexeme()) {
                let distance = self.scopes.len() - 1 - i;
                self.locals.insert(expr.clone(), distance);
            }
        }
    }

    fn declare(&mut self, name: &Token) -> Result {
        if let Some(scope) = self.scopes.last_mut() {
            let lexeme = name.get_lexeme();
            if scope.contains_key(&lexeme) {
                return Err(Error::VariableAlreadyDeclared { name: lexeme });
            }
            scope.insert(lexeme, false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.get_lexeme(), true);
        }
    }

    fn resolve_function(&mut self, declaration: &FunctionDeclaration, fn_type: FnType) -> Result {
        let enclosing_fn = self.current_fn;
        self.current_fn = fn_type;

        self.begin_scope();
        for param in &declaration.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(&declaration.body)?;
        self.end_scope();

        self.current_fn = enclosing_fn;
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
                self.resolve_local(expr, name);
                Ok(())
            }

            Expr::Assign { name, value } => {
                self.visit_expr(value)?;
                self.resolve_local(expr, name);
                Ok(())
            }

            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
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

            Expr::Unary { right, .. } => self.visit_expr(right),

            Expr::Lambda(declaration) => self.resolve_function(declaration, FnType::Function),

            Expr::Get { object, .. } => self.visit_expr(object),

            Expr::Set { object, value, .. } => {
                self.visit_expr(object)?;
                self.visit_expr(value)
            }

            Expr::This(keyword) => {
                if self.current_class == ClassType::None {
                    return Err(Error::ThisOutsideClass);
                }
                self.resolve_local(expr, keyword);
                Ok(())
            }
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
                self.define(name);
                Ok(())
            }

            Stmt::Function(declaration) => {
                self.declare(&declaration.name)?;
                self.define(&declaration.name);
                self.resolve_function(declaration, FnType::Function)
            }

            Stmt::Expression(expr) | Stmt::Print(expr) => self.visit_expr(expr),

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

            Stmt::Return { value, .. } => {
                if self.current_fn == FnType::None {
                    return Err(Error::ReturnOutsideFunction);
                }

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

            Stmt::Class { name, methods } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(name)?;
                self.define(name);

                // Add a scope for class methods
                self.begin_scope();
                // Bind `this` to the class
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert("this".to_string(), true);
                } else {
                    return Err(Error::Internal("No scope".to_string()));
                }

                for method in methods {
                    let fn_type = FnType::Method;
                    self.resolve_function(method, fn_type)?;
                }

                self.end_scope();
                self.current_class = enclosing_class;
                Ok(())
            }
        }
    }
}
