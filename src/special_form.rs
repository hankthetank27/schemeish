use std::iter::Peekable;
use std::rc::Rc;
use std::vec::IntoIter;

use crate::{
    enviroment::EnvRef,
    error::EvalErr,
    evaluator::eval,
    lexer::Token,
    parser::Expr,
    primitives::pair::Pair,
    print::Printable,
    procedure::Compound,
    utils::{GetVals, IterInnerVal, ToExpr},
};

pub trait SpecialForm {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Define {
    identifier: Expr,
    body: Vec<Expr>,
}

impl Define {
    pub fn new(identifier: Expr, body: Vec<Expr>) -> Self {
        Define { identifier, body }
    }
}

impl SpecialForm for Define {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        let mut body = self.body.into_iter();
        match self.identifier {
            //bind var
            Expr::Atom(Token::Symbol(identifier)) => {
                let value = body
                    .next()
                    .ok_or(EvalErr::InvalidArgs("variable has no declared value"))?;

                env.insert_val(identifier.to_string(), eval(value, env)?)
            }
            //bind proc
            Expr::List(args) => {
                let (first, rest) = args.into_iter().get_one_and_rest_or_else(|| {
                    EvalErr::InvalidArgs("'define' procedure. expected parameters and body")
                })?;

                match first {
                    Expr::Atom(Token::Symbol(identifier)) => {
                        let lambda = Lambda::new(Expr::List(rest.collect()), body.collect());
                        env.insert_val(identifier.to_string(), lambda.eval(env)?)
                    }
                    _ => Err(EvalErr::InvalidArgs("expected identifier for procedure")),
                }
            }
            identifier => Err(EvalErr::TypeError("symbol or list", identifier)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    params: Expr,
    body: Vec<Expr>,
}

impl Lambda {
    pub fn new(params: Expr, body: Vec<Expr>) -> Self {
        Lambda { params, body }
    }
}

impl SpecialForm for Lambda {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match self.params {
            Expr::List(first_expr) => {
                let proc_args = first_expr.into_strings()?;
                Ok(Compound::new(self.body, proc_args, env.clone_rc()?).to_expr())
            }
            Expr::EmptyList => Ok(Compound::new(self.body, vec![], env.clone_rc()?).to_expr()),
            first_expr => Err(EvalErr::TypeError("list", first_expr)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    predicate: Expr,
    consequence: Expr,
    alternative: Expr,
}

impl If {
    pub fn new(predicate: Expr, consequence: Expr, alternative: Expr) -> Self {
        If {
            predicate,
            consequence,
            alternative,
        }
    }
}

impl SpecialForm for If {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match eval(self.predicate, env)? {
            Expr::Atom(Token::Boolean(true)) => eval(self.consequence, env),
            Expr::Atom(Token::Boolean(false)) => eval(self.alternative, env),
            pred => Err(EvalErr::TypeError("bool", pred)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cond {
    clauses: Vec<Expr>,
}

impl Cond {
    pub fn new(clauses: Vec<Expr>) -> Self {
        Cond { clauses }
    }
}

impl SpecialForm for Cond {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        eval(cond_to_if(&mut self.clauses.into_iter().peekable())?, env)
    }
}

fn cond_to_if(exprs: &mut Peekable<IntoIter<Expr>>) -> Result<Expr, EvalErr> {
    match exprs.next() {
        Some(expr) => match expr {
            Expr::List(expr) => {
                let (predicate, consequence) = expr.into_iter().get_two_or_else(|| {
                    EvalErr::InvalidArgs("'cond'. clauses expcted two be lists of two values")
                })?;

                if exprs.peek().is_some() {
                    If::new(predicate, consequence, cond_to_if(exprs)?)
                        .to_expr()
                        .into_list()
                } else {
                    match predicate {
                        Expr::Atom(Token::Else) => Ok(consequence),
                        _ => If::new(predicate, consequence, cond_to_if(exprs)?)
                            .to_expr()
                            .into_list(),
                    }
                }
            }
            expr => Err(EvalErr::UnexpectedToken(expr.printable())),
        },
        None => Ok(Expr::EmptyList),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    identifier: Expr,
    value: Expr,
}

impl Assignment {
    pub fn new(identifier: Expr, value: Expr) -> Self {
        Assignment { identifier, value }
    }
}

impl SpecialForm for Assignment {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match self.identifier {
            Expr::Atom(Token::Symbol(identifier)) => {
                env.update_val(identifier.to_string(), eval(self.value, env)?)
            }
            expr => Err(EvalErr::TypeError("symbol", expr)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutCell {
    Car,
    Cdr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutatePair {
    target: Expr,
    value: Expr,
    cell: MutCell,
}

impl MutatePair {
    pub fn new(target: Expr, value: Expr, cell: MutCell) -> Self {
        MutatePair {
            target,
            value,
            cell,
        }
    }
}

//also this is not a special form and can be moved into the pair module

// TODO: UNSAFE!!! really need to think about this more lol. maybe use rc<refcell> but that could
// complicate a lot of other things... particularly printing on lists unless we just clone the list
// and consume the clone to print it which actually is totally fine.
impl SpecialForm for MutatePair {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        match eval(self.target, env)? {
            Expr::Dotted(p) => unsafe {
                match self.cell {
                    MutCell::Car => (*(Rc::into_raw(p) as *mut Pair)).car = eval(self.value, env)?,
                    MutCell::Cdr => (*(Rc::into_raw(p) as *mut Pair)).cdr = eval(self.value, env)?,
                };
                Ok(Expr::EmptyList) //TODO: do i need to create an undefined type?
            },
            expr => Err(EvalErr::TypeError("pair", expr)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct And {
    body: Vec<Expr>,
}

impl And {
    pub fn new(body: Vec<Expr>) -> Self {
        And { body }
    }
}

impl SpecialForm for And {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        for expr in self.body.into_iter() {
            match eval(expr, env)? {
                Expr::Atom(Token::Boolean(n)) if !n => return Ok(n.to_expr()),
                Expr::Atom(Token::Boolean(n)) if n => Ok(()),
                expr => Err(EvalErr::TypeError("boolean", expr)),
            }?;
        }
        Ok(true.to_expr())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Or {
    body: Vec<Expr>,
}

impl Or {
    pub fn new(body: Vec<Expr>) -> Self {
        Or { body }
    }
}

impl SpecialForm for Or {
    fn eval(self, env: &EnvRef) -> Result<Expr, EvalErr> {
        for expr in self.body.into_iter() {
            match eval(expr, env)? {
                Expr::Atom(Token::Boolean(n)) if n => return Ok(n.to_expr()),
                Expr::Atom(Token::Boolean(n)) if !n => Ok(()),
                expr => Err(EvalErr::TypeError("boolean", expr)),
            }?;
        }
        Ok(false.to_expr())
    }
}
