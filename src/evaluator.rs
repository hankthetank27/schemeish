use core::f64;

use crate::{enviroment::Env, lexer::Token, parser::Expr, procedure::Proc};

pub fn eval(exp: &Expr, env: &Env) -> Expr {
    match exp {
        // lookup symbol (variable) value in env
        Expr::Atom(Token::Symbol(name)) => env.get_val(name).unwrap(),

        // self evaluating
        Expr::Atom(val) => Expr::Atom(val.clone()),

        // unsure how to handle this case atm
        Expr::Proc(proc) => Expr::Proc(proc.clone()),

        // procedure
        Expr::List(ls) => {
            let operator = ls.get(0).expect("No operator found");
            match operator {
                Expr::Atom(Token::Symbol(op_name)) => {
                    let args = ls[1..].to_vec(); // clones here

                    // we check here first for special forms return Some
                    // special_form(op_name, args, env)
                    apply(op_name, args, env)
                }
                _ => panic!("No symbol found"),
            }
        }
    }
}

pub fn apply(name: &str, args: Vec<Expr>, env: &Env) -> Expr {
    match name {
        "+" => collect_to_nums(args, env).iter().sum::<f64>().to_num_expr(),
        "-" => {
            let nums = collect_to_nums(args, env);
            nums.iter()
                .skip(1)
                .fold(nums[0], |diff, num| diff - num)
                .to_num_expr()
        }

        //here we will look up procs in env
        _ => panic!("operation not yet implemented"),
    }
}

pub fn specical_form(operator: &str, args: Vec<Expr>, env: Env) -> Option<Expr> {
    match operator {
        "define" => todo!(),
        "lambda" => todo!(),
        _ => None,
    }
}

pub fn collect_to_nums(args: Vec<Expr>, env: &Env) -> Vec<f64> {
    args.iter()
        .map(|expr| match eval(expr, &env) {
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
