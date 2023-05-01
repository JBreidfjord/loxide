use std::fmt;

use crate::loxide::ast::Literal;

use super::{functions::NativeFunction, Error};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
    NativeFunction(NativeFunction),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Self::Nil | Self::Bool(false))
    }

    pub fn type_of(&self) -> String {
        match self {
            Self::Nil => String::from("Nil"),
            Self::Number(_) => String::from("Number"),
            Self::Bool(_) => String::from("Bool"),
            Self::String(_) => String::from("String"),
            Self::NativeFunction(_) => String::from("<native fn>"),
        }
    }
}

impl TryFrom<&Literal> for Value {
    type Error = Error;

    fn try_from(literal: &Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::Nil => Ok(Value::Nil),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Number(n) => Ok(Value::Number(*n)),
            Literal::String(s) => Ok(Value::String(s.to_owned())),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => b.fmt(f),
            Self::Number(n) => n.fmt(f),
            Self::String(s) => write!(f, "{:?}", s),
            Self::NativeFunction(nf) => write!(f, "{}", nf.name),
        }
    }
}
