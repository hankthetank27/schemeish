use std::fmt::Debug;
use std::{error::Error, fmt};

use crate::parser::Expr;
use crate::print::Printable;

#[derive(Debug, Clone)]
pub enum EvalErr {
    InvalidExpr(Expr),
    UnboundVar(String),
    InvalidArgs(&'static str),
    TypeError(&'static str, Expr),
    UnexpectedToken(String),
    MalformedToken(&'static str),
    LexingFailures(Vec<EvalErr>),
    RuntimeException(String),
    MapAsRecoverable,
    UnexpectedEnd,
    NilEnv,
}

impl Error for EvalErr {}

impl fmt::Display for EvalErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ERROR: {}", make_message(self))
    }
}

fn make_message(err: &EvalErr) -> String {
    match err {
        EvalErr::RuntimeException(m) => m.to_owned(),
        EvalErr::UnboundVar(var) => format!("accessing unbound variable {var}"),
        EvalErr::InvalidExpr(expr) => format!("invalid expression {}", expr.printable()),
        EvalErr::InvalidArgs(msg) => format!("invalid argument, {msg}"),
        EvalErr::MalformedToken(msg) => msg.to_string(),
        EvalErr::UnexpectedEnd => "unexpected end of expression".to_string(),
        EvalErr::NilEnv => "inserting value into empty enviroment".to_string(),
        EvalErr::MapAsRecoverable => "recoverable".to_string(),
        EvalErr::UnexpectedToken(msg) => format!("unexpected token {msg}"),
        EvalErr::TypeError(expected, got) => {
            format!("expected {expected}, got {}", got.printable())
        }
        EvalErr::LexingFailures(errs) => {
            let err = errs.iter().fold(String::new(), |errs, err| {
                [errs, format!("-- {}", make_message(err))].join("\n")
            });
            format!("{}{}", "could not parse tokens:", err)
        }
    }
}
