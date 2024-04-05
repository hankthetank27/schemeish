use core::panic;

use crate::{enviroment::EnvRef, evaluator, lexer::Token, parser::Expr, procedure::Proc};

pub fn define(args: &Vec<Expr>, env: EnvRef) -> Expr {
    let identifier = args.get(0).expect("Expected identifier");
    match identifier {
        //bind var
        Expr::Atom(Token::Symbol(identifier)) => {
            let value = args.get(1).expect("Expected value for variable");
            let val_expr = evaluator::eval(value, env.clone_rc());
            env.insert_val(identifier.to_string(), val_expr).unwrap()
        }

        //bind proc
        Expr::List(first_ls) => {
            let rest_ls = args;
            let mut first_ls = first_ls.iter().map(|expr| match expr {
                Expr::Atom(Token::Symbol(name)) => name.to_string(),
                _ => panic!("Expected symbol as parameter, got {:?}", expr),
            });

            let proc_name = first_ls.next().expect("Expected identifier for proc");
            let proc_args = first_ls.collect::<Vec<String>>();
            let proc_body = rest_ls[1..].to_vec();

            let proc = Expr::Proc(Proc::new(proc_body, proc_args, env.clone_rc()));
            env.insert_val(proc_name.to_string(), proc).unwrap()
        }
        _ => panic!("Failed to define {:?}", identifier),
    }
}

pub fn lambda(args: &Vec<Expr>, env: EnvRef) -> Expr {
    let mut args = args.iter();
    let first_ls = args.next().expect("Expected list of parameters");
    match first_ls {
        Expr::List(first_ls) => {
            let first_ls = first_ls.iter().map(|expr| match expr {
                Expr::Atom(Token::Symbol(name)) => name.to_string(),
                _ => panic!("Expected symbol as parameter, got {:?}", expr),
            });

            let proc_args = first_ls.collect::<Vec<String>>();
            let proc_body = args.map(|a| a.to_owned()).collect();

            Expr::Proc(Proc::new(proc_body, proc_args, env.clone_rc()))
        }
        _ => panic!("Failed to define lambda. Expected list, got:{:?}", first_ls),
    }
}

pub fn if_statement(args: &Vec<Expr>, env: EnvRef) -> Expr {
    let predicate = evaluator::eval(args.get(0).expect("Expected predicate"), env.clone_rc());
    match predicate {
        Expr::Atom(Token::Boolean(true)) => {
            evaluator::eval(args.get(1).expect("Expected consequence"), env)
        }
        Expr::Atom(Token::Boolean(false)) => {
            evaluator::eval(args.get(2).expect("Expected alternative"), env)
        }
        _ => panic!("Expected boolean, got {:?}", predicate),
    }
}
