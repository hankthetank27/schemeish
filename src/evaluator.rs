use std::panic;

use crate::{enviroment::EnvRef, lexer::Token, parser::Expr, procedure::Proc};

pub fn eval(expr: &Expr, env: EnvRef) -> Expr {
    match expr.to_owned() {
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
            let args: Vec<Expr> = ls.collect();
            match operation {
                Expr::Atom(Token::Symbol(_)) => apply(operation, args, env),
                Expr::List(_) => apply(operation, args, env),
                _ => panic!("Evaluated invalid expression, {:?}", &expr),
            }
        }
    }
}

pub fn apply(operation: Expr, args: Vec<Expr>, env: EnvRef) -> Expr {
    let operation = eval(&operation, env.clone_rc());
    match operation {
        Expr::Proc(proc) => match proc {
            Proc::Primitive(proc) => proc.call(args, env),
            Proc::Compound(proc) => proc.call(eval_list(&args, env)),
        },
        _ => panic!("Expected procedure, got {:?}", operation),
    }
}

pub fn eval_list(epxrs: &Vec<Expr>, env: EnvRef) -> Vec<Expr> {
    epxrs
        .iter()
        .map(|expr| eval(expr, env.clone_rc()))
        .collect()
}
