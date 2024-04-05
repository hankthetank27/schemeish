use std::panic;

use crate::{enviroment::EnvRef, lexer::Token, parser::Expr, primitives, special_forms};

pub fn eval(expr: &Expr, env: EnvRef) -> Expr {
    match expr {
        // variable lookup
        Expr::Atom(Token::Symbol(identifier)) => {
            env.get_val(identifier).expect("Access unbound variable")
        }
        // self evaluating
        Expr::Atom(val) => Expr::Atom(val.clone()),
        // procedure
        Expr::List(ls) => {
            let operation = ls.get(0).expect("No operator found");
            match operation {
                Expr::Atom(Token::Symbol(op_id)) => {
                    let args = ls[1..].to_vec(); // clones here
                    try_special(op_id, &args, env.clone_rc())
                        .or_else(|| try_primitive(op_id, &args, env.clone_rc()))
                        .unwrap_or_else(|| apply(operation, &args, env))
                }
                Expr::List(_) => {
                    let args = ls[1..].to_vec(); // clones here
                    apply(operation, &args, env)
                }
                _ => panic!("Evaluated invalid expression, {:?}", expr),
            }
        }

        // unsure how to handle this case atm
        Expr::Proc(proc) => Expr::Proc(proc.clone()),
    }
}

pub fn apply(operation: &Expr, args: &Vec<Expr>, env: EnvRef) -> Expr {
    let operation = eval(operation, env.clone_rc());
    match operation {
        Expr::Proc(proc) => proc.call(eval_list(args, env)),
        _ => panic!("Expected procedure, got {:?}", operation),
    }
}

pub fn eval_list(epxrs: &Vec<Expr>, env: EnvRef) -> Vec<Expr> {
    epxrs
        .iter()
        .map(|expr| eval(expr, env.clone_rc()))
        .collect()
}

fn try_special(operation: &str, args: &Vec<Expr>, env: EnvRef) -> Option<Expr> {
    match operation.trim() {
        "define" => Some(special_forms::define(args, env)),
        "lambda" => Some(special_forms::lambda(args, env)),
        "if" => Some(special_forms::if_statement(args, env)),
        _ => None,
    }
}

fn try_primitive(operation: &str, args: &Vec<Expr>, env: EnvRef) -> Option<Expr> {
    match operation.trim() {
        "+" => Some(primitives::add(args, env)),
        "-" => Some(primitives::subtract(args, env)),
        "*" => Some(primitives::multiply(args, env)),
        "/" => Some(primitives::divide(args, env)),
        "=" => Some(primitives::equality(args, env)),
        ">" => Some(primitives::greater_than(args, env)),
        ">=" => Some(primitives::greater_than_or_eq(args, env)),
        "<" => Some(primitives::less_than(args, env)),
        "<=" => Some(primitives::less_than_or_eq(args, env)),
        _ => None,
    }
}
