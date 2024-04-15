use std::iter::Peekable;
use std::vec::IntoIter;

use crate::error::EvalErr;
use crate::lexer::Token;
use crate::parser::Expr;
use crate::primitives::pair::Pair;
use crate::procedure::Proc;

pub trait IterInnerVal {
    fn into_nums(self) -> Result<Vec<f64>, EvalErr>;
    fn into_strings(self) -> Result<Vec<String>, EvalErr>;
}

impl IterInnerVal for Vec<Expr> {
    fn into_nums(self) -> Result<Vec<f64>, EvalErr> {
        self.into_iter()
            .map(|expr| match expr {
                Expr::Atom(Token::Number(n)) => Ok(n),
                _ => Err(EvalErr::TypeError(("number", expr))),
            })
            .collect()
    }

    fn into_strings(self) -> Result<Vec<String>, EvalErr> {
        self.into_iter()
            .map(|expr| match expr {
                Expr::Atom(Token::Symbol(name)) => Ok(name.to_string()),
                _ => Err(EvalErr::TypeError(("symbol", expr))),
            })
            .collect()
    }
}

pub trait HasNext<I: Iterator> {
    fn has_next(self) -> Option<Peekable<I>>;
}

impl<I, T> HasNext<I> for Peekable<I>
where
    I: Iterator<Item = T>,
{
    fn has_next(mut self) -> Option<Peekable<I>> {
        self.peek().is_some().then(|| self)
    }
}

pub trait ToExpr {
    fn to_expr(self) -> Expr;
}

impl ToExpr for f64 {
    fn to_expr(self) -> Expr {
        Expr::Atom(Token::Number(self))
    }
}

impl ToExpr for Proc {
    fn to_expr(self) -> Expr {
        Expr::Proc(self)
    }
}

impl ToExpr for bool {
    fn to_expr(self) -> Expr {
        Expr::Atom(Token::Boolean(self))
    }
}

impl ToExpr for Vec<Expr> {
    fn to_expr(self) -> Expr {
        Expr::List(self)
    }
}

impl ToExpr for Pair {
    fn to_expr(self) -> Expr {
        Expr::Dotted(self)
    }
}

pub trait GetVals {
    fn get_one(&mut self) -> Result<Expr, EvalErr>;
    fn get_two(&mut self) -> Result<(Expr, Expr), EvalErr>;
    fn get_three(&mut self) -> Result<(Expr, Expr, Expr), EvalErr>;
    fn get_one_and_rest(self) -> Result<(Expr, IntoIter<Expr>), EvalErr>;
}

// this is not the right error message
impl GetVals for IntoIter<Expr> {
    fn get_one(&mut self) -> Result<Expr, EvalErr> {
        self.next()
            .ok_or_else(|| EvalErr::InvalidArgs("not enough arguments"))
    }

    fn get_two(&mut self) -> Result<(Expr, Expr), EvalErr> {
        let err = || EvalErr::InvalidArgs("not enough arguments");
        let first = self.next().ok_or_else(err)?;
        let second = self.next().ok_or_else(err)?;
        Ok((first, second))
    }

    fn get_three(&mut self) -> Result<(Expr, Expr, Expr), EvalErr> {
        let err = || EvalErr::InvalidArgs("not enough arguments");
        let first = self.next().ok_or_else(err)?;
        let second = self.next().ok_or_else(err)?;
        let third = self.next().ok_or_else(err)?;
        Ok((first, second, third))
    }

    fn get_one_and_rest(mut self) -> Result<(Expr, IntoIter<Expr>), EvalErr> {
        let err = || EvalErr::InvalidArgs("not enough arguments");
        let first = self.next().ok_or_else(err)?;
        Ok((first, self))
    }
}
