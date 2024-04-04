use core::panic;

use crate::{enviroment::EnvRef, evaluator, lexer::Token, parser::Expr};

pub fn add(args: &Vec<Expr>, env: EnvRef) -> Expr {
    evaluator::eval_list(args, env)
        .to_nums()
        .sum::<f64>()
        .to_expr()
}

pub fn multiply(args: &Vec<Expr>, env: EnvRef) -> Expr {
    evaluator::eval_list(args, env)
        .to_nums()
        .product::<f64>()
        .to_expr()
}

pub fn subtract(args: &Vec<Expr>, env: EnvRef) -> Expr {
    let mut nums = evaluator::eval_list(args, env).to_nums();
    if let Some(first) = nums.next() {
        nums.fold(first, |diff, num| diff - num).to_expr()
    } else {
        panic!("Procedure requires at least one argument")
    }
}

pub fn divide(args: &Vec<Expr>, env: EnvRef) -> Expr {
    let mut nums = evaluator::eval_list(args, env).to_nums();
    if let Some(first) = nums.next() {
        nums.fold(first, |diff, num| diff / num).to_expr()
    } else {
        panic!("Procedure requires at least one argument")
    }
}

pub fn equality(args: &Vec<Expr>, env: EnvRef) -> Expr {
    let nums: Vec<f64> = evaluator::eval_list(args, env).to_nums().collect();
    if let Some(first) = nums.get(0) {
        nums.iter().all(|e| e == first).to_expr()
    } else {
        panic!("Procedure requires at least one argument")
    }
}

trait Collect {
    fn to_nums(self) -> impl Iterator<Item = f64>;
}

impl Collect for Vec<Expr> {
    fn to_nums(self) -> impl Iterator<Item = f64> {
        self.into_iter().map(|expr| match expr {
            Expr::Atom(Token::Number(n)) => n,
            _ => panic!("Expected number, got {:?}", expr),
        })
    }
}

trait ToExpr {
    fn to_expr(self) -> Expr;
}

impl ToExpr for f64 {
    fn to_expr(self) -> Expr {
        Expr::Atom(Token::Number(self))
    }
}

impl ToExpr for bool {
    fn to_expr(self) -> Expr {
        Expr::Atom(Token::Boolean(self))
    }
}
