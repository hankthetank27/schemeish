// use core::cell::RefCell;
// use std::rc::Rc;

use core::iter::Peekable;
use std::vec::IntoIter;

use crate::error::EvalErr;
use crate::evaluator::Args;
use crate::parser::Expr;
use crate::primitives::utils::{GetVals, HasNext};

use super::utils::ToExpr;

#[derive(Debug, Clone, PartialEq)]
pub struct Pair {
    // car: Rc<RefCell<Expr>>,
    // cdr: Rc<RefCell<Expr>>,
    pub car: Box<Expr>,
    pub cdr: Box<Expr>,
}

impl Pair {
    fn new(car: Expr, cdr: Expr) -> Pair {
        Pair {
            car: Box::new(car),
            cdr: Box::new(cdr),
        }
    }

    fn car(self) -> Expr {
        *self.car
    }

    fn cdr(self) -> Expr {
        *self.cdr
    }
}

pub fn cons(args: Args) -> Result<Expr, EvalErr> {
    let (first, second) = args.eval()?.into_iter().get_two()?;
    match second {
        Expr::List(mut ls) => {
            ls.insert(0, first);
            Ok(ls.to_expr())
        }
        x @ Expr::Dotted(_) | x @ Expr::Atom(_) | x @ Expr::Proc(_) | x @ Expr::EmptyList => {
            Ok(Pair::new(first, x).to_expr())
        }
    }
}

pub fn car(args: Args) -> Result<Expr, EvalErr> {
    let expr = args.eval()?.into_iter().get_one()?;
    match expr {
        Expr::List(ls) => ls.into_iter().get_one().or_else(|_| Ok(Expr::EmptyList)),
        Expr::Dotted(p) => Ok(p.car()),
        Expr::EmptyList => Err(EvalErr::InvalidArgs("cannot access car of empty list")),
        x => Err(EvalErr::TypeError(("list", x))),
    }
}

pub fn cdr(args: Args) -> Result<Expr, EvalErr> {
    let expr = args.eval()?.into_iter().get_one()?;
    match expr {
        Expr::List(ls) => {
            let (_, rest) = ls.into_iter().get_one_and_rest()?;
            match rest.peekable().has_next() {
                Some(ls) => Ok(ls.collect::<Vec<Expr>>().to_expr()),
                None => Ok(Expr::EmptyList),
            }
        }
        Expr::Dotted(p) => Ok(p.cdr()),
        Expr::EmptyList => Err(EvalErr::InvalidArgs("cannot access cdr of empty list")),
        x => Err(EvalErr::TypeError(("list", x))),
    }
}

pub fn list(args: Args) -> Result<Expr, EvalErr> {
    fn map_to_list(el: Expr, mut ls: Peekable<IntoIter<Expr>>) -> Expr {
        let next = match ls.peek() {
            Some(_) => map_to_list(ls.next().unwrap(), ls),
            None => Expr::EmptyList,
        };
        Pair::new(el, next).to_expr()
    }
    let (first, rest) = args.eval()?.into_iter().get_one_and_rest()?;
    Ok(map_to_list(first, rest.peekable()))
}

pub fn null_check(args: Args) -> Result<Expr, EvalErr> {
    match args.eval()?.into_iter().get_one()? {
        Expr::EmptyList => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}

pub fn pair_check(args: Args) -> Result<Expr, EvalErr> {
    match args.eval()?.into_iter().get_one()? {
        Expr::Dotted(_) => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}
