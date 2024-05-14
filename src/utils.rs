use std::iter::Peekable;
use std::rc::Rc;
use std::vec::IntoIter;

use crate::error::EvalErr;
use crate::lexer::Token;
use crate::parser::Expr;
use crate::primitives::pair::Pair;
use crate::procedure::Proc;
use crate::special_form::And;
use crate::special_form::Assignment;
use crate::special_form::Begin;
use crate::special_form::Cond;
use crate::special_form::Define;
use crate::special_form::If;
use crate::special_form::Lambda;
use crate::special_form::Let;
use crate::special_form::Or;
use crate::special_form::SpecialForm;

pub trait IterInnerVal {
    fn into_nums(self) -> Result<Vec<f64>, EvalErr>;
    fn into_strings(self) -> Result<Vec<String>, EvalErr>;
}

impl<T> IterInnerVal for T
where
    T: IntoIterator<Item = Expr>,
{
    fn into_nums(self) -> Result<Vec<f64>, EvalErr> {
        self.into_iter()
            .map(|expr| match expr {
                Expr::Atom(Token::Number(n)) => Ok(n),
                _ => Err(EvalErr::TypeError("number", expr)),
            })
            .collect()
    }

    fn into_strings(self) -> Result<Vec<String>, EvalErr> {
        self.into_iter()
            .map(|expr| match expr {
                Expr::Atom(Token::Symbol(name)) => Ok(name.to_string()),
                _ => Err(EvalErr::TypeError("symbol", expr)),
            })
            .collect()
    }
}

pub trait SoftIter<I>
where
    I: Iterator,
{
    fn has_next(self) -> Option<Peekable<I>>;
    fn take_until<F>(&mut self, pred: F) -> IntoIter<I::Item>
    where
        F: Fn(&I::Item) -> bool;
}

impl<I> SoftIter<I> for Peekable<I>
where
    I: Iterator,
{
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

pub trait GetVals<F, T>
where
    F: Fn() -> EvalErr,
{
    fn get_one_or_else(&mut self, err: F) -> Result<T, EvalErr>;
    fn get_two_or_else(&mut self, err: F) -> Result<(T, T), EvalErr>;
    fn get_three_or_else(&mut self, err: F) -> Result<(T, T, T), EvalErr>;
    fn get_one_and_rest_or_else(self, err: F) -> Result<(T, IntoIter<T>), EvalErr>;
}

impl<F, T> GetVals<F, T> for IntoIter<T>
where
    F: Fn() -> EvalErr,
{
    fn get_one_or_else(&mut self, err: F) -> Result<T, EvalErr> {
        self.next().ok_or_else(err)
    }

    fn get_two_or_else(&mut self, err: F) -> Result<(T, T), EvalErr> {
        let first = self.next().ok_or_else(&err)?;
        let second = self.next().ok_or_else(&err)?;
        Ok((first, second))
    }

    fn get_three_or_else(&mut self, err: F) -> Result<(T, T, T), EvalErr> {
        let first = self.next().ok_or_else(&err)?;
        let second = self.next().ok_or_else(&err)?;
        let third = self.next().ok_or_else(&err)?;
        Ok((first, second, third))
    }

    fn get_one_and_rest_or_else(mut self, err: F) -> Result<(T, IntoIter<T>), EvalErr> {
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
        Expr::Proc(Box::new(self))
    }
}

impl ToExpr for Vec<Expr> {
    fn to_expr(self) -> Expr {
        Expr::List(self)
    }
}

impl ToExpr for Pair {
    fn to_expr(self) -> Expr {
        Expr::Dotted(Rc::new(self))
    }
}

impl ToExpr for If {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::If(self)))
    }
}

impl ToExpr for Lambda {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::Lambda(self)))
    }
}

impl ToExpr for Define {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::Define(self)))
    }
}

impl ToExpr for Assignment {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::Assignment(self)))
    }
}

impl ToExpr for And {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::And(self)))
    }
}

impl ToExpr for Or {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::Or(self)))
    }
}

impl ToExpr for Cond {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::Cond(self)))
    }
}

impl ToExpr for Begin {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::Begin(self)))
    }
}

impl ToExpr for Let {
    fn to_expr(self) -> Expr {
        Expr::SpecialForm(Box::new(SpecialForm::Let(self)))
    }
}
