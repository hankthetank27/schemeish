use std::fmt::Debug;
use std::{error::Error, fmt};

use crate::parser::Expr;

#[derive(Debug)]
pub enum EvalErr {
    InvalidExpr(Expr),
    UnboundVar(String),
    InvalidArgs(&'static str),
    TypeError((&'static str, Expr)),
    NilEnv,
}

impl Error for EvalErr {}

impl fmt::Display for EvalErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            EvalErr::UnboundVar(var) => format!("accessing unbound variable {var}"),
            EvalErr::InvalidExpr(expr) => format!("invalid expression {:?}", expr),
            EvalErr::InvalidArgs(msg) => format!("invalid argument, {msg}"),
            EvalErr::TypeError((expected, got)) => format!("expected {expected}, got {:?}", got),
            EvalErr::NilEnv => "inserting value into empty enviroment".to_string(),
        };
        write!(f, "ERROR: {}", err_msg)
    }
}

#[derive(Debug)]
pub enum ParseErr {
    UnexpectedToken(String),
    UnexpectedEnd,
    MalformedToken(&'static str),
}

impl Error for ParseErr {}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            ParseErr::UnexpectedToken(msg) => msg.to_string(),
            ParseErr::MalformedToken(msg) => msg.to_string(),
            ParseErr::UnexpectedEnd => "unexpected end of expression".to_string(),
        };
        write!(f, "ERROR: {}", err_msg)
    }
}
