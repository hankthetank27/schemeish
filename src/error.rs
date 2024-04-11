use crate::parser::Expr;
use std::{error::Error, fmt};

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
            EvalErr::UnboundVar(var) => format!("unbound variable {var}"),
            EvalErr::InvalidExpr(expr) => format!("invalid expression {:?}", expr),
            EvalErr::InvalidArgs(msg) => format!("invalid argument, {msg}"),
            EvalErr::TypeError((expected, got)) => format!("expected {expected}, got {:?}", got),
            EvalErr::NilEnv => "inserting value into empty enviroment".to_string(),
        };
        write!(f, "ERROR: {}", err_msg)
    }
}
