use crate::{error::EvalErr, lexer::Token, parser::Expr, procedure::Proc};
use std::iter::Peekable;

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

impl<I> HasNext<I> for Peekable<I>
where
    I: Iterator<Item = f64>,
{
    fn has_next(mut self) -> Option<Peekable<I>> {
        match self.peek().is_some() {
            true => Some(self),
            false => None,
        }
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
