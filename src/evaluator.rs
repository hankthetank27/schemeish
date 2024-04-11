use std::vec::IntoIter;

use crate::{enviroment::EnvRef, error::EvalErr, lexer::Token, parser::Expr, procedure::Proc};

pub fn eval(expr: Expr, env: &EnvRef) -> Result<Expr, EvalErr> {
    match expr {
        // variable lookup
        Expr::Atom(Token::Symbol(ref identifier)) => env.get_val(identifier),
        // self evaluating
        x @ Expr::Atom(_) | x @ Expr::Proc(_) => Ok(x),
        // procedure
        Expr::List(ls) => {
            let mut ls = ls.into_iter();
            let operation = ls.next().expect("No operator found");
            let args = Args::new(ls.collect(), env);
            match operation {
                Expr::Atom(Token::Symbol(_)) => apply(operation, args),
                Expr::List(_) => apply(operation, args),
                operation => Err(EvalErr::TypeError(("symbol or list", operation))),
            }
        }
    }
}

pub fn apply(operation: Expr, args: Args) -> Result<Expr, EvalErr> {
    match eval(operation, &args.env())? {
        Expr::Proc(proc) => match proc {
            Proc::Primitive(proc) => proc.call(args),
            Proc::Compound(proc) => proc.call(args.eval()?),
        },
        op => Err(EvalErr::TypeError(("procedure", op))),
    }
}

pub struct Args {
    args: Vec<Expr>,
    env: EnvRef,
}

impl Args {
    pub fn new(args: Vec<Expr>, env: &EnvRef) -> Args {
        Args {
            args,
            env: env.clone_rc(),
        }
    }

    pub fn eval(self) -> Result<Vec<Expr>, EvalErr> {
        self.args
            .into_iter()
            .map(|expr| eval(expr, &self.env))
            .collect()
    }

    pub fn into_iter(self) -> IntoIter<Expr> {
        self.args.into_iter()
    }

    pub fn env(&self) -> EnvRef {
        self.env.clone_rc()
    }
}
