use crate::{lexer::Token, parser::Expr, procedure::Proc};
use std::iter::Peekable;

pub trait IterInnerVal {
    fn into_nums(self) -> impl Iterator<Item = f64>;
    fn into_strings(self) -> impl Iterator<Item = String>;
}

impl IterInnerVal for Vec<Expr> {
    fn into_nums(self) -> impl Iterator<Item = f64> {
        self.into_iter().map(|expr| match expr {
            Expr::Atom(Token::Number(n)) => n,
            _ => panic!("Expected number, got {:?}", expr),
        })
    }

    fn into_strings(self) -> impl Iterator<Item = String> {
        self.into_iter().map(|expr| match expr {
            Expr::Atom(Token::Symbol(name)) => name.to_string(),
            _ => panic!("Expected symbol as parameter, got {:?}", expr),
        })
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
