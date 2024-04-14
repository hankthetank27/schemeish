use crate::error::EvalErr;
use crate::evaluator::Args;
use crate::parser::Expr;
use crate::primitives::utils::{GetVals, HasNext};

use super::utils::ToExpr;

// I think we need to make dotted list / runtime lists seperate (linked list)
pub fn cons(args: Args) -> Result<Expr, EvalErr> {
    let (first, second) = args.eval()?.into_iter().get_two()?;
    match second {
        Expr::List(mut ls) => {
            ls.insert(0, first);
            Ok(Expr::List(ls))
        }
        x @ Expr::Atom(_) | x @ Expr::Proc(_) | x @ Expr::EmptyList => {
            Ok(Expr::List(vec![first, x]))
        }
    }
}

pub fn car(args: Args) -> Result<Expr, EvalErr> {
    let expr = args.eval()?.into_iter().get_one()?;
    match expr {
        Expr::List(ls) => ls.into_iter().get_one().or_else(|_| Ok(Expr::EmptyList)),
        e @ Expr::EmptyList => Ok(e),
        x => Err(EvalErr::TypeError(("list", x))),
    }
}

pub fn cdr(args: Args) -> Result<Expr, EvalErr> {
    let expr = args.eval()?.into_iter().get_one()?;
    println!("{:?}", expr);
    match expr {
        Expr::List(ls) => {
            let (_, rest) = ls.into_iter().get_one_and_rest()?;
            match rest.peekable().has_next() {
                Some(ls) => Ok(ls.collect::<Vec<Expr>>().to_expr()),
                None => Ok(Expr::EmptyList),
            }
        }
        e @ Expr::EmptyList => Ok(e),
        x => Err(EvalErr::TypeError(("list", x))),
    }
}

pub fn list(args: Args) -> Result<Expr, EvalErr> {
    Ok(Expr::List(args.eval()?))
}

pub fn null_check(args: Args) -> Result<Expr, EvalErr> {
    match args.eval()?.into_iter().get_one()? {
        Expr::EmptyList => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}
