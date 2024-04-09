use core::panic;

use crate::{
    evaluator::{eval, Args},
    lexer::Token,
    parser::Expr,
    primitives::utils::{IterInnerVal, ToExpr},
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
            let val_expr = eval(value, &env);
            env.insert_val(identifier.to_string(), val_expr).unwrap()
        }

        //bind proc
        Expr::List(first_expr) => {
            let mut first_expr = first_expr.into_strings();
            let proc_name = first_expr.next().expect("Expected identifier for proc");
            let proc_args = first_expr.collect::<Vec<String>>();
            let proc_body = args.collect();

            let proc = Compound::new(proc_body, proc_args, env.clone_rc()).to_expr();
            env.insert_val(proc_name.to_string(), proc).unwrap()
        }
        identifier => panic!("Failed to define {:?}", identifier),
    }
}

pub fn lambda(args: Args) -> Expr {
    let env = args.env();
    let mut args = args.into_iter();
    let first_expr = args.next().expect("Expected list of parameters");
    match first_expr {
        Expr::List(first_expr) => {
            let proc_args = first_expr.into_strings().collect();
            let proc_body = args.map(|a| a.to_owned()).collect();

            Compound::new(proc_body, proc_args, env.clone_rc()).to_expr()
        }
        first_expr => panic!(
            "Failed to define lambda. Expected list, got:{:?}",
            first_expr
        ),
    }
}

pub fn if_statement(args: Args) -> Expr {
    let env = args.env();
    let mut args = args.into_iter();
    let predicate = args.next().expect("Expected predicate");
    let consequence = args.next().expect("Expected consequence");
    let alternative = args.next().expect("Expected alternative");
    match eval(predicate, &env) {
        Expr::Atom(Token::Boolean(true)) => eval(consequence, &env),
        Expr::Atom(Token::Boolean(false)) => eval(alternative, &env),
        pred => panic!("Expected boolean, got {:?}", pred),
    }
}
