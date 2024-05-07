use crate::{
    error::EvalErr,
    evaluator::Args,
    lexer::Token,
    parser::Expr,
    utils::{GetVals, ToExpr},
};

pub fn not(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .get_one_or_else(|| EvalErr::InvalidArgs("'not'. expected argument"))?
    {
        Expr::Atom(Token::Boolean(b)) => Ok((!b).to_expr()),
        _ => Ok(false.to_expr()),
    }
}
