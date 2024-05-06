use std::rc::Rc;
use std::vec;

use core::iter::Peekable;

use crate::error::EvalErr;
use crate::evaluator::Args;
use crate::parser::Expr;
use crate::utils::{GetVals, ToExpr};

#[derive(Debug, Clone, PartialEq)]
pub struct Pair {
    pub car: Expr,
    pub cdr: Expr,
}

impl Pair {
    pub fn new(car: Expr, cdr: Expr) -> Pair {
        Pair { car, cdr }
    }

    // printing methods
    pub fn try_list(&self) -> MaybeList {
        match self.check_if_list() {
            Some(ls) => MaybeList::List(ls),
            None => MaybeList::Pair(self),
        }
    }

    // ^^
    fn check_if_list(&self) -> Option<PairList> {
        match &self.cdr {
            Expr::Dotted(next) => {
                let cdr = next.check_if_list()?;
                let node = Some(Box::new(Node::new(&self.car, cdr)));
                Some(node)
            }
            Expr::EmptyList => {
                let node = Some(Box::new(Node::new(&self.car, None)));
                Some(node)
            }
            _ => None,
        }
    }
}

type PairList<'a> = Option<Box<Node<'a>>>;

pub enum MaybeList<'a> {
    List(PairList<'a>),
    Pair(&'a Pair),
}

pub struct Node<'a> {
    pub car: &'a Expr,
    pub cdr: PairList<'a>,
}

impl<'a> Node<'a> {
    fn new(car: &'a Expr, cdr: PairList<'a>) -> Self {
        Node { car, cdr }
    }

    pub fn iter(&self) -> Iter {
        Iter { next: Some(self) }
    }
}

pub struct Iter<'a> {
    next: Option<&'a Node<'a>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Expr;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|next| {
            self.next = next.cdr.as_deref();
            next.car
        })
    }
}

pub fn cons(args: Args) -> Result<Expr, EvalErr> {
    let (first, second) = args
        .into_iter()
        .get_two_or_else(|| EvalErr::InvalidArgs("'cons'. expected two arguments."))?;

    Ok(Pair::new(first, second).to_expr())
}

pub fn car(args: Args) -> Result<Expr, EvalErr> {
    let expr = args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'car'. expected argument"))?;

    match expr {
        Expr::Dotted(p) => Ok(p.as_ref().car.clone()),
        Expr::EmptyList => Err(EvalErr::InvalidArgs("cannot access car of empty list")),
        x => Err(EvalErr::TypeError("pair", x)),
    }
}

pub fn cdr(args: Args) -> Result<Expr, EvalErr> {
    let expr = args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'cdr'. expected argument"))?;

    match expr {
        Expr::Dotted(p) => Ok(p.as_ref().cdr.clone()),
        Expr::EmptyList => Err(EvalErr::InvalidArgs("cannot access cdr of empty list")),
        x => Err(EvalErr::TypeError("pair", x)),
    }
}

// TODO: UNSAFE!!! really need to think about this more lol. maybe use rc<refcell> but that could
// complicate a lot of other things... particularly printing on lists unless we just clone the list
// and consume the clone to print it which actually is totally fine.
pub fn set_car(args: Args) -> Result<Expr, EvalErr> {
    let (target, source) = args
        .into_iter()
        .get_two_or_else(|| EvalErr::InvalidArgs("'car'. expected argument"))?;

    match target {
        Expr::Dotted(p) => unsafe {
            (*(Rc::into_raw(p) as *mut Pair)).car = source;
            Ok(Expr::Void)
        },
        expr => Err(EvalErr::TypeError("pair", expr)),
    }
}

pub fn set_cdr(args: Args) -> Result<Expr, EvalErr> {
    let (target, source) = args
        .into_iter()
        .get_two_or_else(|| EvalErr::InvalidArgs("'cdr'. expected argument"))?;

    match target {
        Expr::Dotted(p) => unsafe {
            (*(Rc::into_raw(p) as *mut Pair)).cdr = source;
            Ok(Expr::Void)
        },
        expr => Err(EvalErr::TypeError("pair", expr)),
    }
}

pub fn list(args: Args) -> Result<Expr, EvalErr> {
    fn map_to_list(el: Expr, mut ls: Peekable<vec::IntoIter<Expr>>) -> Expr {
        let next = match ls.peek() {
            Some(_) => map_to_list(ls.next().unwrap(), ls),
            None => Expr::EmptyList,
        };
        Pair::new(el, next).to_expr()
    }

    let (first, rest) = args
        .into_iter()
        .get_one_and_rest_or_else(|| EvalErr::InvalidArgs("'list'. expected arguments"))?;

    Ok(map_to_list(first, rest.peekable()))
}

pub fn null_check(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'nil?'. expected argument"))?
    {
        Expr::EmptyList => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}

pub fn pair_check(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'pair?'. expected argument"))?
    {
        Expr::Dotted(_) => Ok(true.to_expr()),
        _ => Ok(false.to_expr()),
    }
}
