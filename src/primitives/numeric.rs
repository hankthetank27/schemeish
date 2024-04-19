use core::f64;

use crate::{
    error::EvalErr,
    evaluator::Args,
    parser::Expr,
    utils::{HasNext, IterInnerVal, ToExpr},
};

pub fn add(args: Args) -> Result<Expr, EvalErr> {
    Ok(args.eval()?.into_nums()?.iter().sum::<f64>().to_expr())
}

pub fn multiply(args: Args) -> Result<Expr, EvalErr> {
    Ok(args.eval()?.into_nums()?.iter().product::<f64>().to_expr())
}

pub fn subtract(args: Args) -> Result<Expr, EvalErr> {
    let mut nums = args.eval()?.into_nums()?.into_iter();
    match nums.next() {
        Some(first) => Ok(nums.fold(first, |diff, num| diff - num).to_expr()),
        None => Err(EvalErr::InvalidArgs(
            "Procedure requires at least one argument",
        )),
    }
}

pub fn divide(args: Args) -> Result<Expr, EvalErr> {
    let mut nums = args.eval()?.into_nums()?.into_iter();
    match nums.next() {
        Some(first) => Ok(nums.fold(first, |quot, num| quot / num).to_expr()),
        None => Err(EvalErr::InvalidArgs(
            "Procedure requires at least one argument",
        )),
    }
}

pub fn equality(args: Args) -> Result<Expr, EvalErr> {
    let nums: Vec<f64> = args.eval()?.into_nums()?;
    match nums.first() {
        Some(first) => Ok(nums.iter().all(|num| num == first).to_expr()),
        None => Err(EvalErr::InvalidArgs(
            "Procedure requires at least one argument",
        )),
    }
}

pub fn greater_than(args: Args) -> Result<Expr, EvalErr> {
    cmp_first_to_rest(args, |first, rest| first > rest)
}

pub fn greater_than_or_eq(args: Args) -> Result<Expr, EvalErr> {
    cmp_first_to_rest(args, |first, rest| first >= rest)
}

pub fn less_than(args: Args) -> Result<Expr, EvalErr> {
    cmp_first_to_rest(args, |first, rest| first < rest)
}

pub fn less_than_or_eq(args: Args) -> Result<Expr, EvalErr> {
    cmp_first_to_rest(args, |first, rest| first <= rest)
}

fn cmp_first_to_rest<F>(args: Args, cmp: F) -> Result<Expr, EvalErr>
where
    F: Fn(f64, f64) -> bool,
{
    let mut nums = args.eval()?.into_nums()?.into_iter().peekable();
    let err = EvalErr::InvalidArgs("Procedure requires at least two arguments");
    match nums.next() {
        Some(first) => {
            let sum_rest = nums.has_next().ok_or(err)?.sum();
            Ok(cmp(first, sum_rest).to_expr())
        }
        None => Err(err),
    }
}
