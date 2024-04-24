use std::iter::Peekable;
use std::vec::IntoIter;

use crate::error::EvalErr;
use crate::lexer::Token;
use crate::parser::Expr;
use crate::primitives::pair::Pair;
use crate::procedure::Proc;
use crate::special_form::Assignment;
use crate::special_form::Define;
use crate::special_form::If;
use crate::special_form::Lambda;

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

pub trait SoftIter<I: Iterator> {
    fn has_next(self) -> Option<Peekable<I>>;
    fn take_until<F>(&mut self, pred: F) -> IntoIter<I::Item>
    where
        F: Fn(&I::Item) -> bool;
}

impl<I: Iterator> SoftIter<I> for Peekable<I> {
    fn has_next(mut self) -> Option<Peekable<I>> {
        self.peek().is_some().then_some(self)
    }

    fn take_until<F>(&mut self, pred: F) -> IntoIter<I::Item>
    where
        F: Fn(&I::Item) -> bool,
    {
        let mut new = vec![];
        while self.peek().map_or(false, &pred) {
            new.push(self.next().unwrap())
        }
        new.into_iter()
    }
}

pub trait GetVals<F>
where
    F: Fn() -> EvalErr,
{
    fn get_one_or_else(&mut self, err: F) -> Result<Expr, EvalErr>;
    fn get_two_or_else(&mut self, err: F) -> Result<(Expr, Expr), EvalErr>;
    fn get_three_or_else(&mut self, err: F) -> Result<(Expr, Expr, Expr), EvalErr>;
    fn get_one_and_rest_or_else(self, err: F) -> Result<(Expr, IntoIter<Expr>), EvalErr>;
}

// this is not the right error message
// let err = || EvalErr::InvalidArgs("not enough arguments");
impl<F> GetVals<F> for IntoIter<Expr>
where
    F: Fn() -> EvalErr,
{
    fn get_one_or_else(&mut self, err: F) -> Result<Expr, EvalErr> {
        self.next().ok_or_else(err)
    }

    fn get_two_or_else(&mut self, err: F) -> Result<(Expr, Expr), EvalErr> {
        let first = self.next().ok_or_else(&err)?;
        let second = self.next().ok_or_else(&err)?;
        Ok((first, second))
    }

    fn get_three_or_else(&mut self, err: F) -> Result<(Expr, Expr, Expr), EvalErr> {
        let first = self.next().ok_or_else(&err)?;
        let second = self.next().ok_or_else(&err)?;
        let third = self.next().ok_or_else(&err)?;
        Ok((first, second, third))
    }

    fn get_one_and_rest_or_else(mut self, err: F) -> Result<(Expr, IntoIter<Expr>), EvalErr> {
        let first = self.next().ok_or_else(&err)?;
        Ok((first, self))
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

impl ToExpr for &str {
    fn to_expr(self) -> Expr {
        Expr::Atom(Token::Symbol(self.to_string()))
    }
}

impl ToExpr for bool {
    fn to_expr(self) -> Expr {
        Expr::Atom(Token::Boolean(self))
    }
}

impl ToExpr for Proc {
    fn to_expr(self) -> Expr {
        Expr::Proc(self)
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

impl ToExpr for If {
    fn to_expr(self) -> Expr {
        Expr::If(Box::new(self))
    }
}

impl ToExpr for Lambda {
    fn to_expr(self) -> Expr {
        Expr::Lambda(Box::new(self))
    }
}

impl ToExpr for Define {
    fn to_expr(self) -> Expr {
        Expr::Define(Box::new(self))
    }
}

impl ToExpr for Assignment {
    fn to_expr(self) -> Expr {
        Expr::Assignment(Box::new(self))
    }
}
