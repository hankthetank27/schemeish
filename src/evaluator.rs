use std::panic;

use crate::{enviroment::EnvRef, lexer::Token, parser::Expr, primitives, special_forms};

pub fn eval(expr: &Expr, env: EnvRef) -> Expr {
    match expr {
        // lookup symbol (variable) value in env
        Expr::Atom(Token::Symbol(identifier)) => {
            env.get_val(identifier).expect("Access unbound variable")
        }

        // self evaluating
        Expr::Atom(val) => Expr::Atom(val.clone()),

        // procedure
        Expr::List(ls) => {
            let operation = ls.get(0).expect("No operator found");
            match operation {
                // TODO: consolidate. I'm not a big fan of how this is written
                // regarding calls to apply/special_form but it gets the job
                // done for now.
                Expr::Atom(Token::Symbol(op_id)) => {
                    let args = ls[1..].to_vec(); // clones here
                    match special(op_id, &args, env.clone_rc()) {
                        Some(expr) => expr,
                        None => apply(operation, &args, env),
                    }
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
        Expr::Proc(proc) => proc.call(eval_list(args, env.clone_rc())),
        _ => panic!("Expected procedure, got {:?}", operation),
    }
}

pub fn eval_list(epxrs: &Vec<Expr>, env: EnvRef) -> Vec<Expr> {
    epxrs
        .iter()
        .map(|expr| eval(expr, env.clone_rc()))
        .collect()
}

fn special(operation: &str, args: &Vec<Expr>, env: EnvRef) -> Option<Expr> {
    match operation {
        "define" => Some(special_forms::define(args, env.clone_rc())),
        "lambda" => Some(special_forms::lambda(args, env.clone_rc())),
        "+" => Some(primitives::add(args, env.clone_rc())),
        "-" => Some(primitives::subtract(args, env.clone_rc())),
        _ => None,
    }
}
