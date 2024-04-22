use std::fmt::Debug;
use std::{error::Error, fmt};

use crate::parser::Expr;

#[derive(Debug, Clone)]
pub enum EvalErr {
    InvalidExpr(Expr),
    UnboundVar(String),
    InvalidArgs(&'static str),
    TypeError((&'static str, Expr)),
    UnexpectedToken(String),
    MalformedToken(&'static str),
    MapAsRecoverable,
    UnexpectedEnd,
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
            EvalErr::UnexpectedToken(msg) => format!("unexpected token {msg}"),
            EvalErr::MalformedToken(msg) => msg.to_string(),
            EvalErr::UnexpectedEnd => "unexpected end of expression".to_string(),
            EvalErr::NilEnv => "inserting value into empty enviroment".to_string(),
            EvalErr::MapAsRecoverable => "recoverable".to_string(),
        };
        write!(f, "ERROR: {}", err_msg)
    }
}
