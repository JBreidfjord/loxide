use std::fmt;

use ordered_float::OrderedFloat;

use crate::loxide::ast::Literal;

use super::{
    classes::{Class, Instance},
    functions::{Function, NativeFunction},
    Error,
};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(OrderedFloat<f64>),
    Bool(bool),
    String(String),
    NativeFunction(NativeFunction),
    Function(Function),
    Class(Class),
    Instance(Instance),
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
            Self::Function(_) => String::from("<fn>"),
            Self::Class(_) => String::from("<class>"),
            Self::Instance(_) => String::from("<instance>"),
        }
    }

    pub fn try_into_class(self) -> Result<Class, Error> {
        match self {
            Self::Class(class) => Ok(class),
            _ => Err(Error::ConversionError {
                from: self,
                to: String::from("<class>"),
            }),
        }
    }

    pub fn try_into_function(self) -> Result<Function, Error> {
        match self {
            Self::Function(func) => Ok(func),
            _ => Err(Error::ConversionError {
                from: self,
                to: String::from("<fn>"),
            }),
        }
    }

    pub fn try_into_instance(self) -> Result<Instance, Error> {
        match self {
            Self::Instance(instance) => Ok(instance),
            _ => Err(Error::ConversionError {
                from: self,
                to: String::from("<instance>"),
            }),
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
            Literal::String(s) => Ok(Value::String(s.clone())),
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
            Self::String(s) => write!(f, "{}", s),
            Self::NativeFunction(nf) => write!(f, "{:?}", nf),
            Self::Function(func) => write!(f, "{:?}", func),
            Self::Class(class) => write!(f, "{:?}", class),
            Self::Instance(instance) => write!(f, "{:?}", instance),
        }
    }
}
