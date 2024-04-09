use core::{f64, panic};

use crate::{
    evaluator::Args,
    parser::Expr,
    primitives::utils::{HasNext, IterInnerVal, ToExpr},
};

pub fn add(args: Args) -> Expr {
    args.eval().into_nums().sum::<f64>().to_expr()
}

pub fn multiply(args: Args) -> Expr {
    args.eval().into_nums().product::<f64>().to_expr()
}

pub fn subtract(args: Args) -> Expr {
    let mut nums = args.eval().into_nums();
    match nums.next() {
        Some(first) => nums.fold(first, |diff, num| diff - num).to_expr(),
        None => panic!("Procedure requires at least one argument"),
    }
}

pub fn divide(args: Args) -> Expr {
    let mut nums = args.eval().into_nums();
    match nums.next() {
        Some(first) => nums.fold(first, |quot, num| quot / num).to_expr(),
        None => panic!("Procedure requires at least one argument"),
    }
}

pub fn equality(args: Args) -> Expr {
    let nums: Vec<f64> = args.eval().into_nums().collect();
    match nums.get(0) {
        Some(first) => nums.iter().all(|num| num == first).to_expr(),
        None => panic!("Procedure requires at least one argument"),
    }
}

pub fn greater_than(args: Args) -> Expr {
    cmp_first_to_rest(args, |first, rest| first > rest)
}

pub fn greater_than_or_eq(args: Args) -> Expr {
    cmp_first_to_rest(args, |first, rest| first >= rest)
}

pub fn less_than(args: Args) -> Expr {
    cmp_first_to_rest(args, |first, rest| first < rest)
}

pub fn less_than_or_eq(args: Args) -> Expr {
    cmp_first_to_rest(args, |first, rest| first <= rest)
}

fn cmp_first_to_rest<F>(args: Args, cmp: F) -> Expr
where
    F: Fn(f64, f64) -> bool,
{
    let mut nums = args.eval().into_nums().peekable();
    match nums.next() {
        Some(first) => {
            let sum_rest = nums
                .has_next()
                .expect("Procedure requires at least two arguments")
                .sum();
            cmp(first, sum_rest).to_expr()
        }
        None => panic!("Procedure requires at least two argument"),
    }
}
