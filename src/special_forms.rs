use core::panic;

use crate::{
    evaluator::{eval, Args},
    lexer::Token,
    parser::Expr,
    procedure::Compound,
};

pub fn define(args: Args) -> Expr {
    let env = args.env();
    let mut args = args.into_iter();
    let identifier = args.next().expect("Expected identifier");
    match identifier {
        //bind var
        Expr::Atom(Token::Symbol(identifier)) => {
            let value = args.next().expect("Expected value for variable");
            let val_expr = eval(&value, &env);
            env.insert_val(identifier.to_string(), val_expr).unwrap()
        }

        //bind proc
        Expr::List(first_ls) => {
            let mut first_ls = first_ls.iter().map(|expr| match expr {
                Expr::Atom(Token::Symbol(name)) => name.to_string(),
                _ => panic!("Expected symbol as parameter, got {:?}", expr),
            });

            let proc_name = first_ls.next().expect("Expected identifier for proc");
            let proc_args = first_ls.collect::<Vec<String>>();
            let proc_body = args.collect();

            let proc = Expr::Proc(Compound::new(proc_body, proc_args, env.clone_rc()));
            env.insert_val(proc_name.to_string(), proc).unwrap()
        }
        identifier => panic!("Failed to define {:?}", identifier),
    }
}

pub fn lambda(args: Args) -> Expr {
    let env = args.env();
    let mut args = args.into_iter();
    let first_ls = args.next().expect("Expected list of parameters");
    match first_ls {
        Expr::List(first_ls) => {
            let first_ls = first_ls.iter().map(|expr| match expr {
                Expr::Atom(Token::Symbol(name)) => name.to_string(),
                _ => panic!("Expected symbol as parameter, got {:?}", expr),
            });

            let proc_args = first_ls.collect::<Vec<String>>();
            let proc_body = args.map(|a| a.to_owned()).collect();

            Expr::Proc(Compound::new(proc_body, proc_args, env.clone_rc()))
        }
        first_ls => panic!("Failed to define lambda. Expected list, got:{:?}", first_ls),
    }
}

pub fn if_statement(args: Args) -> Expr {
    let env = args.env();
    let mut args = args.into_iter();
    let first_ls = args.next().expect("Expected list of parameters");
    match eval(&first_ls, &env) {
        Expr::Atom(Token::Boolean(true)) => eval(&args.next().expect("Expected consequence"), &env),
        Expr::Atom(Token::Boolean(false)) => {
            eval(&args.nth(1).expect("Expected alternative"), &env)
        }
        pred => panic!("Expected boolean, got {:?}", pred),
    }
}
