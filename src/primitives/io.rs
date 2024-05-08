use crate::{
    error::EvalErr, evaluator::Args, lexer::Token, parser::Expr, print::Print, utils::GetVals,
};

pub fn display(args: Args) -> Result<Expr, EvalErr> {
    let expr = args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'display'. expected argument"))?;
    expr.print();
    Ok(expr)
}

pub fn error(args: Args) -> Result<Expr, EvalErr> {
    let expr = args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'error'. expected argument"))?;
    match expr {
        Expr::Atom(Token::Str(msg)) => Err(EvalErr::RuntimeException(msg)),
        _ => Err(EvalErr::InvalidArgs(
            "'error'. expected string as argument.",
        )),
    }
}
