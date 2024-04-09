use std::panic;
use std::vec::IntoIter;

use crate::{enviroment::EnvRef, lexer::Token, parser::Expr, procedure::Proc};

pub fn eval(expr: &Expr, env: &EnvRef) -> Expr {
    let expr = expr.to_owned();
    let env = env.clone_rc();
    match expr {
        // variable lookup
        Expr::Atom(Token::Symbol(identifier)) => env
            .get_val(&identifier)
            .unwrap_or_else(|| panic!("Attempted accessing unbound variable {:?}", &identifier)),
        // self evaluating
        x @ Expr::Atom(_) | x @ Expr::Proc(_) => x,
        // procedure
        Expr::List(ls) => {
            let mut ls = ls.into_iter();
            let operation = ls.next().expect("No operator found");
            let args = Args::new(ls.collect(), &env);
            match operation {
                Expr::Atom(Token::Symbol(_)) => apply(operation, args),
                Expr::List(_) => apply(operation, args),
                operation => panic!("Evaluated invalid expression, {:?}", operation),
            }
        }
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

    pub fn eval(&self) -> Vec<Expr> {
        self.args
            .iter()
            .map(|expr| eval(&expr, &self.env))
            .collect()
    }

    pub fn into_iter(self) -> IntoIter<Expr> {
        self.args.into_iter()
    }

    pub fn env(&self) -> EnvRef {
        self.env.clone_rc()
    }
}

pub fn apply(operation: Expr, args: Args) -> Expr {
    let operation = eval(&operation, &args.env());
    match operation {
        Expr::Proc(proc) => match proc {
            Proc::Primitive(proc) => proc.call(args),
            Proc::Compound(proc) => proc.call(args.eval()),
        },
        _ => panic!("Expected procedure, got {:?}", operation),
    }
}
