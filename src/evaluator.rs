use crate::enviroment::EnvRef;
use crate::error::EvalErr;
use crate::lexer::Token;
use crate::parser::Expr;
use crate::procedure::Proc;
use crate::{special_form::SpecialForm, utils::GetVals};

pub fn eval(expr: Expr, env: &EnvRef) -> Result<Expr, EvalErr> {
    match expr {
        // variable lookup
        Expr::Atom(Token::Symbol(ref identifier)) => env.get_val(identifier),
        // procedure
        Expr::List(ls) => {
            let (op, args) = ls
                .into_iter()
                .get_one_and_rest_or_else(|| EvalErr::InvalidArgs("expected operation"))?;
            let args = Args::new(args.collect(), env)?;

            match op {
                Expr::If(if_x) => if_x.eval(env),
                Expr::Cond(cond_x) => cond_x.eval(env),
                Expr::Define(def_x) => def_x.eval(env),
                Expr::Assignment(ass_x) => ass_x.eval(env),
                Expr::And(and_x) => and_x.eval(env),
                Expr::Begin(beg_x) => beg_x.eval(env),
                Expr::Let(let_x) => let_x.eval(env),
                Expr::Lambda(lam_x) => lam_x.eval(env),
                Expr::Or(or_x) => or_x.eval(env),
                Expr::List(_) => apply(op, args),
                Expr::Atom(Token::Symbol(_)) => apply(op, args),
                op => Err(EvalErr::TypeError("procedure or special form", op)),
            }
        }
        // self evaluating
        Expr::Quoted(x) => Ok(*x),
        x @ Expr::Atom(_) | x @ Expr::Dotted(_) | x @ Expr::EmptyList => Ok(x),
        x => Err(EvalErr::TypeError("expression", x)),
    }
}

pub fn apply(op: Expr, args: Args) -> Result<Expr, EvalErr> {
    match eval(op, &args.env()?)? {
        Expr::Proc(proc) => match proc {
            Proc::Primitive(proc) => proc.call(args.eval()?),
            Proc::Compound(proc) => proc.call(args.eval()?),
        },
        op => Err(EvalErr::TypeError("procedure", op)),
    }
}

pub struct Args {
    args: Vec<Expr>,
    env: EnvRef,
}

impl Args {
    pub fn new(args: Vec<Expr>, env: &EnvRef) -> Result<Args, EvalErr> {
        Ok(Args {
            args,
            env: env.clone_rc()?,
        })
    }

    pub fn eval(mut self) -> Result<Args, EvalErr> {
        let evaled = self
            .args
            .into_iter()
            .map(|expr| eval(expr, &self.env))
            .collect::<Result<Vec<Expr>, EvalErr>>()?;

        self.args = evaled;
        Ok(self)
    }

    pub fn env(&self) -> Result<EnvRef, EvalErr> {
        self.env.clone_rc()
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

impl IntoIterator for Args {
    type Item = Expr;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}
