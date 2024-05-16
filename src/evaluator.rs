use crate::enviroment::EnvRef;
use crate::error::EvalErr;
use crate::lexer::Token;
use crate::parser::Expr;
use crate::procedure::Proc;
use crate::{special_form::Eval, utils::OwnIterVals};

pub fn eval(expr: &Expr, env: &EnvRef) -> Result<Expr, EvalErr> {
    match expr.clone() {
        // variable lookup
        Expr::Atom(Token::Symbol(ref identifier)) => env.get_val(identifier),
        // procedure
        Expr::Call(ls) => {
            let (op, args) = ls
                .into_iter()
                .own_one_and_rest_or_else(|| EvalErr::InvalidArgs("expected operation"))?;
            let args = Args::new(args.collect(), env)?;
            match op {
                Expr::SpecialForm(x) => x.eval(env),
                _ => apply(op, args),
            }
        }
        // self evaluating
        Expr::Quoted(x) => Ok(*x),
        x @ Expr::Atom(_) | x @ Expr::Pair(_) | x @ Expr::EmptyList | x @ Expr::Void => Ok(x),
        x => Err(EvalErr::TypeError("expression", x)),
    }
}

pub fn apply(op: Expr, args: Args) -> Result<Expr, EvalErr> {
    let env = &args.env()?;
    match eval(&op, env)? {
        Expr::Proc(proc) => match proc.as_ref() {
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
            .map(|expr| eval(&expr, &self.env))
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
