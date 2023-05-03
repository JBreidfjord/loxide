use std::{collections::HashMap, time};

use ordered_float::OrderedFloat;
use thiserror::Error;

use self::{
    classes::Class,
    environment::Environment,
    functions::{Callable, Function, NativeFunction},
    value::Value,
};

use super::{
    ast::{Expr, Stmt, Visitor},
    token::Token,
    token_type::TokenType,
};

mod classes;
mod environment;
pub mod functions;
mod value;

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "Operator `{operator}` expected one of: [{}], found {} of type {}.",
        .expected.join(", "),
        .found,
        .found.type_of()
    )]
    InvalidOperand {
        operator: TokenType,
        expected: Vec<String>,
        found: Value,
    },

    #[error("Unsupported unary operator `{operator}` on type {}.", .value.type_of())]
    UnsupportedUnary { operator: TokenType, value: Value },

    #[error(
        "Unsupported binary operator `{operator}` on types {} and {}.",
        .left.type_of(),
        .right.type_of()
    )]
    UnsupportedBinary {
        operator: TokenType,
        left: Value,
        right: Value,
    },

    #[error("Undefined variable {name}.")]
    UndefinedVariable { name: String },

    #[error("Break statement outside of loop.")]
    Break,

    #[error("Cannot call non-callable value of type `{}`.", .value.type_of())]
    NotCallable { value: Value },

    #[error("Expected {expected} arguments but found {found}.")]
    InvalidArgumentCount { expected: usize, found: usize },

    #[error(transparent)]
    SystemTimeError(#[from] time::SystemTimeError),

    #[error("Return statement outside of function.")]
    Return(Value),

    #[error("Tried to access property `{property}` on non-object `{value}` of type `{}`.", .value.type_of())]
    PropertyOnNonObject { property: String, value: Value },

    #[error("Undefined property `{property}` on object `{value}`.")]
    UndefinedProperty { property: String, value: Value },

    #[error("Superclass {value} must be a class.")]
    SuperclassNotAClass { value: Value },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Interpreter {
    environment: Environment,
    globals: Environment,
    locals: HashMap<Expr, usize>,
}

impl Interpreter {
    pub fn new(locals: HashMap<Expr, usize>) -> Self {
        let mut globals = Environment::global();

        // Define the clock native function
        globals.define(
            "clock".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "clock".to_string(),
                arity: 0,
                function: |_, _| {
                    Ok(Value::Number(OrderedFloat(
                        time::SystemTime::now()
                            .duration_since(time::UNIX_EPOCH)?
                            .as_secs_f64(),
                    )))
                },
            }),
        );

        Self {
            environment: globals.clone(),
            globals,
            locals,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        for stmt in statements {
            self.visit_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn execute_block(&mut self, statements: &[Stmt], environment: Environment) -> Result<()> {
        let current = self.environment.clone(); // Store current environment

        // Set environment for the block and visit each statement
        self.environment = environment;
        let result = statements.iter().try_for_each(|stmt| self.visit_stmt(stmt));

        self.environment = current; // Restore current environment

        result // Return result of block
    }

    fn lookup_variable(&self, name: &Token, expr: &Expr) -> Result<Value> {
        // Look up the variable in the local or global environment
        let value = if let Some(distance) = self.locals.get(expr) {
            self.environment.lookup_at(*distance, name.get_lexeme())
        } else {
            self.globals.lookup(name.get_lexeme())
        };

        value.ok_or(Error::UndefinedVariable {
            name: name.get_lexeme(),
        })
    }
}

impl Visitor<Result<Value>, Result<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.visit_expr(expr)?;
            }

            Stmt::Print(expr) => println!("{}", self.visit_expr(expr)?),

            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.visit_expr(expr)?,
                    None => Value::Nil,
                };
                self.environment.define(name.get_lexeme(), value);
            }

            Stmt::Block(statements) => self.execute_block(statements, self.environment.nest())?,

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.visit_expr(condition)?;

                if condition.is_truthy() {
                    self.visit_stmt(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.visit_stmt(else_branch)?;
                }
            }

            Stmt::While { condition, body } => {
                while self.visit_expr(condition)?.is_truthy() {
                    match self.visit_stmt(body) {
                        Err(Error::Break) => break,
                        result => result?,
                    };
                }
            }

            Stmt::Break => return Err(Error::Break),

            Stmt::Function(declaration) => {
                let function = Function::new(declaration.clone(), self.environment.clone());
                self.environment
                    .define(declaration.name.get_lexeme(), Value::Function(function));
            }

            Stmt::Return { value, .. } => {
                let value = match value {
                    Some(expr) => self.visit_expr(expr)?,
                    None => Value::Nil,
                };
                return Err(Error::Return(value));
            }

            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let superclass = if let Some(superclass) = superclass {
                    let superclass = self.visit_expr(superclass)?;
                    match superclass {
                        Value::Class(class) => Ok(Some(Box::new(Value::Class(class)))),
                        _ => Err(Error::SuperclassNotAClass { value: superclass }),
                    }
                } else {
                    Ok(None)
                }?;

                self.environment.define(name.get_lexeme(), Value::Nil);

                let mut class_methods = HashMap::new();
                for method in methods {
                    let function = if method.name.get_lexeme() == "init" {
                        Function::new_init(method.clone(), self.environment.clone())
                    } else {
                        Function::new(method.clone(), self.environment.clone())
                    };
                    class_methods.insert(method.name.get_lexeme(), Value::Function(function));
                }

                let class = Class {
                    name: name.get_lexeme(),
                    superclass,
                    methods: class_methods,
                };
                self.environment
                    .assign(name.get_lexeme(), Value::Class(class));
            }
        }

        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(literal) => Value::try_from(literal),

            Expr::Grouping(expr) => self.visit_expr(expr),

            Expr::Unary { operator, right } => {
                let right = self.visit_expr(right)?;

                match operator.get_token_type() {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => invalid_operand_error(operator, &["Number"], right),
                    },
                    TokenType::Bang => Ok(Value::Bool(!right.is_truthy())),
                    op => Err(Error::UnsupportedUnary {
                        operator: op,
                        value: right,
                    }),
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.visit_expr(left)?;
                let right = self.visit_expr(right)?;

                match operator.get_token_type() {
                    TokenType::Minus => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Slash => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Star => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Plus => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => {
                            Ok(Value::String(format!("{}{}", l, r)))
                        }
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (Value::String(_), right) => {
                            invalid_operand_error(operator, &["String"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number", "String"], left),
                    },
                    TokenType::Greater => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::Less => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                        (Value::Number(_), right) => {
                            invalid_operand_error(operator, &["Number"], right)
                        }
                        (left, _) => invalid_operand_error(operator, &["Number"], left),
                    },
                    TokenType::BangEqual => Ok(Value::Bool(left != right)),
                    TokenType::EqualEqual => Ok(Value::Bool(left == right)),
                    _ => Err(Error::UnsupportedBinary {
                        operator: operator.get_token_type(),
                        left,
                        right,
                    }),
                }
            }

            Expr::Variable(name) | Expr::This(name) => self.lookup_variable(name, expr),

            Expr::Assign { name, value } => {
                let value = self.visit_expr(value)?;
                let result = if let Some(distance) = self.locals.get(expr) {
                    self.environment
                        .assign_at(*distance, name.get_lexeme(), value.clone())
                } else {
                    self.globals.assign(name.get_lexeme(), value.clone())
                };

                if result {
                    Ok(value)
                } else {
                    Err(Error::UndefinedVariable {
                        name: name.get_lexeme(),
                    })
                }
            }

            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.visit_expr(left)?;

                // Short-circuit based on the operator
                if operator.get_token_type() == TokenType::Or {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else if !left.is_truthy() {
                    return Ok(left);
                }

                self.visit_expr(right)
            }

            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                let callee = self.visit_expr(callee)?;

                let callable: Box<dyn Callable> = match callee {
                    Value::NativeFunction(function) => Box::new(function),
                    Value::Function(function) => Box::new(function),
                    Value::Class(class) => Box::new(class),
                    _ => return Err(Error::NotCallable { value: callee }),
                };

                let arguments = arguments
                    .iter()
                    .map(|argument| self.visit_expr(argument))
                    .collect::<Result<Vec<_>>>()?;

                if arguments.len() != callable.arity() {
                    return Err(Error::InvalidArgumentCount {
                        expected: callable.arity(),
                        found: arguments.len(),
                    });
                }

                callable.call(self, arguments)
            }

            Expr::Lambda(lambda) => Ok(Value::Function(Function::new(
                lambda.clone(),
                self.environment.clone(),
            ))),

            Expr::Get { object, name } => {
                let object = self.visit_expr(object)?;

                if let Value::Instance(ref instance) = object {
                    instance.get(name).ok_or(Error::UndefinedProperty {
                        property: name.get_lexeme(),
                        value: object,
                    })
                } else {
                    Err(Error::PropertyOnNonObject {
                        property: name.get_lexeme(),
                        value: object,
                    })
                }
            }

            Expr::Set {
                object,
                name,
                value,
            } => {
                let object = self.visit_expr(object)?;

                if let Value::Instance(mut instance) = object {
                    let value = self.visit_expr(value)?;
                    instance.set(name, value.clone());
                    Ok(value)
                } else {
                    Err(Error::PropertyOnNonObject {
                        property: name.get_lexeme(),
                        value: object,
                    })
                }
            }
        }
    }
}

fn invalid_operand_error<V, S: ToString>(
    operator: &Token,
    expected: &[S],
    found: Value,
) -> Result<V> {
    Err(Error::InvalidOperand {
        operator: operator.get_token_type(),
        expected: expected.iter().map(ToString::to_string).collect(),
        found,
    })
}
