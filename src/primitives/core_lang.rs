use crate::{error::EvalErr, evaluator, parser::Expr, procedure::Proc, utils::OwnIterVals};

use super::pair::OwnPtrInner;

pub fn apply(args: evaluator::Args) -> Result<Expr, EvalErr> {
    let env = args.env()?;
    let (op, args) = args.into_iter().own_two_or_else(|| {
        EvalErr::InvalidArgs("'apply'. expected operation and list of arguments")
    })?;

    let args = match args {
        Expr::Call(ls) => Ok(ls),
        Expr::Pair(p) => Ok(p.inner_to_owned().into_iter().collect()),
        _ => Err(EvalErr::InvalidArgs(
            "'apply'. expected list as second argument",
        )),
    }?;

    let args = evaluator::Args::new(args, &env)?;

    match op {
        Expr::Proc(proc) => match proc.as_ref() {
            Proc::Primitive(proc) => proc.call(args.eval()?),
            Proc::Compound(proc) => proc.call(args.eval()?),
        },
        op => Err(EvalErr::TypeError("procedure", op)),
    }
}
