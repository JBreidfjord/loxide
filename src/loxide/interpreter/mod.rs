use std::time;

use thiserror::Error;

use self::{
    environment::Environment,
    functions::{Callable, NativeFunction},
    value::Value,
};

use super::{
    ast::{Expr, Stmt, Visitor},
    token::Token,
    token_type::TokenType,
};

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

    #[error("Cannot call non-callable value of type {}.", .value.type_of())]
    NotCallable { value: Value },

    #[error("Expected {expected} arguments but found {found}.")]
    InvalidArgumentCount { expected: usize, found: usize },

    #[error(transparent)]
    SystemTimeError(#[from] time::SystemTimeError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Interpreter {
    globals: Environment,
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::global();

        // Define the clock native function
        globals.define(
            "clock".to_string(),
            Value::NativeFunction(NativeFunction {
                name: "clock".to_string(),
                arity: 0,
                function: |_, _| {
                    Ok(Value::Number(
                        time::SystemTime::now()
                            .duration_since(time::UNIX_EPOCH)?
                            .as_secs_f64(),
                    ))
                },
            }),
        );

        Self {
            globals: globals.clone(),
            environment: globals,
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
                todo!("Interpret function statement")
            }
        }

        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(literal) => {
                let value = Value::try_from(literal)?;
                Ok(value)
            }

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

            Expr::Variable(name) => {
                self.environment
                    .lookup(name.get_lexeme())
                    .ok_or(Error::UndefinedVariable {
                        name: name.get_lexeme(),
                    })
            }

            Expr::Assign { name, value } => {
                let value = self.visit_expr(value)?;
                if self.environment.assign(name.get_lexeme(), value.clone()) {
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