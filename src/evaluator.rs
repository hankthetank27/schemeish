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

        // unsure how to handle this case atm
        Expr::Proc(proc) => Expr::Proc(proc.clone()),

        // procedure
        Expr::List(ls) => {
            let operator = ls.get(0).expect("No operator found");
            match operator {
                Expr::Atom(Token::Symbol(op_name)) => {
                    let args = &ls[1..].to_vec(); // clones here
                    if let Some(res) = special_form(op_name, args, env.clone_rc()) {
                        res
                    } else {
                        apply(op_name, args, env)
                    }
                }
                _ => panic!("No symbol found"),
            }
        }
    }
}

pub fn apply(op_name: &str, args: &Vec<Expr>, env: EnvRef) -> Expr {
    match op_name {
        "+" => collect_to_nums(args, env).iter().sum::<f64>().to_num_expr(),
        "-" => {
            let nums = collect_to_nums(args, env);
            nums.iter()
                .skip(1)
                .fold(nums[0], |diff, num| diff - num)
                .to_num_expr()
        }
        // call proc
        _ => {
            let proc = env.get_val(op_name).expect("Access unbound procedure");

            // eval args (should prob make this its own fn)
            let args = args
                .iter()
                .map(|arg| eval(arg, env.clone_rc()))
                .collect::<Vec<Expr>>();

            match proc {
                Expr::Proc(proc) => proc.call(args),
                _ => panic!("Expected procedure"),
            }
        }
    }
}

pub fn special_form(operator: &str, args: &Vec<Expr>, env: EnvRef) -> Option<Expr> {
    match operator {
        "define" => {
            let identifier = args.get(0).expect("Expected identifier for variable");
            match identifier {
                //bind var
                Expr::Atom(Token::Symbol(identifier)) => {
                    let value = args.get(1).expect("Expected value for variable");
                    let val_expr = eval(value, env.clone_rc());
                    env.insert_val(identifier.to_string(), val_expr)
                }

                //bind proc
                Expr::List(ls) => {
                    let mut str_ls = ls.iter().map(|expr| match expr {
                        Expr::Atom(Token::Symbol(name)) => name.to_string(),
                        _ => panic!("Expected symbol as parameter"),
                    });

                    let proc_name = str_ls.next().expect("Expected identifier for proc");
                    let proc_args = str_ls.collect::<Vec<String>>();
                    let proc_body = args[1..].to_vec();

                    let proc = Expr::Proc(Proc::new(proc_body, proc_args, env.clone_rc()));
                    env.insert_val(proc_name.to_string(), proc)
                }
                _ => None,
            }
        }
        // "lambda" => {
        //     let proc_args = args.iter().next().unwrap();
        //     let proc_body = args.iter().next().unwrap();
        //     // Proc::new(proc_body, proc_args, env);
        //     todo!()
        // }
        _ => None,
    }
}

pub fn collect_to_nums(args: &Vec<Expr>, env: EnvRef) -> Vec<f64> {
    args.iter()
        .map(|expr| match eval(expr, env.clone_rc()) {
            Expr::Atom(Token::Number(n)) => n,
            _ => panic!("attempted to perform arithmetic on a non-number value"),
        })
        .collect()
}

pub trait ToExpr {
    fn to_num_expr(self) -> Expr;
}

impl ToExpr for f64 {
    fn to_num_expr(self) -> Expr {
        Expr::Atom(Token::Number(self))
    }
}
