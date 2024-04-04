use core::f64;
use std::panic;

use crate::{enviroment::EnvRef, lexer::Token, parser::Expr, procedure::Proc};

pub fn eval(exp: &Expr, env: EnvRef) -> Expr {
    match exp {
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
                    match special_form(op_id, &args, env.clone_rc()) {
                        Some(expr) => expr,
                        None => apply(operation, &args, env),
                    }
                }
                // we would call
                Expr::List(_) => {
                    let args = ls[1..].to_vec(); // clones here
                    apply(operation, &args, env)
                }
                _ => panic!("No symbol found"),
            }
        }

        // unsure how to handle this case atm
        Expr::Proc(proc) => Expr::Proc(proc.clone()),
    }
}

pub fn apply(operation: &Expr, args: &Vec<Expr>, env: EnvRef) -> Expr {
    match eval(operation, env.clone_rc()) {
        Expr::Proc(proc) => proc.call(eval_list(args, env.clone_rc())),
        _ => panic!("Expected procedure"),
    }
}

pub fn eval_list(epxrs: &Vec<Expr>, env: EnvRef) -> Vec<Expr> {
    epxrs
        .iter()
        .map(|expr| eval(expr, env.clone_rc()))
        .collect()
}

pub fn special_form(operation: &str, args: &Vec<Expr>, env: EnvRef) -> Option<Expr> {
    match operation {
        "define" => {
            let identifier = args.get(0).expect("Expected identifier");
            match identifier {
                //bind var
                Expr::Atom(Token::Symbol(identifier)) => {
                    let value = args.get(1).expect("Expected value for variable");
                    let val_expr = eval(value, env.clone_rc());
                    env.insert_val(identifier.to_string(), val_expr)
                }

                //bind proc
                Expr::List(first_ls) => {
                    let rest_ls = args;
                    let mut first_ls = first_ls.iter().map(|expr| match expr {
                        Expr::Atom(Token::Symbol(name)) => name.to_string(),
                        _ => panic!("Expected symbol as parameter"),
                    });

                    let proc_name = first_ls.next().expect("Expected identifier for proc");
                    let proc_args = first_ls.collect::<Vec<String>>();
                    let proc_body = rest_ls[1..].to_vec();

                    let proc = Expr::Proc(Proc::new(proc_body, proc_args, env.clone_rc()));
                    env.insert_val(proc_name.to_string(), proc)
                }
                _ => None,
            }
        }
        "lambda" => {
            let mut args = args.iter();
            let first_ls = args.next().expect("Expected list of parameters");
            match first_ls {
                Expr::List(first_ls) => {
                    let first_ls = first_ls.iter().map(|expr| match expr {
                        Expr::Atom(Token::Symbol(name)) => name.to_string(),
                        _ => panic!("Expected symbol as parameter"),
                    });

                    let proc_args = first_ls.collect::<Vec<String>>();
                    let proc_body = args.map(|a| a.to_owned()).collect();

                    Some(Expr::Proc(Proc::new(proc_body, proc_args, env.clone_rc())))
                }
                _ => None,
            }
        }
        "+" => Some(eval_list(args, env).to_nums().sum::<f64>().to_num_expr()),
        "-" => {
            let nums: Vec<f64> = eval_list(args, env).to_nums().collect();
            Some(
                nums.iter()
                    .skip(1)
                    .fold(nums[0], |diff, num| diff - num)
                    .to_num_expr(),
            )
        }
        _ => None,
    }
}

//TODO: ???
pub fn def_proc(params: &Vec<Expr>, env: EnvRef) -> Proc {
    todo!()
}

trait Collect {
    fn to_nums(&self) -> impl Iterator<Item = f64>;
}

impl Collect for Vec<Expr> {
    fn to_nums(&self) -> impl Iterator<Item = f64> {
        self.iter().map(|expr| match expr {
            Expr::Atom(Token::Number(n)) => *n,
            _ => panic!("attempted to perform arithmetic on a non-number value"),
        })
    }
}

pub trait ToExpr {
    fn to_num_expr(self) -> Expr;
}

impl ToExpr for f64 {
    fn to_num_expr(self) -> Expr {
        Expr::Atom(Token::Number(self))
    }
}
