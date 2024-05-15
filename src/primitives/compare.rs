use crate::{
    error::EvalErr,
    evaluator::Args,
    lexer::Token,
    parser::Expr,
    utils::{OwnIterVals, ToExpr},
};

pub fn not(args: Args) -> Result<Expr, EvalErr> {
    match args
        .into_iter()
        .own_one_or_else(|| EvalErr::InvalidArgs("'not'. expected argument"))?
    {
        Expr::Atom(Token::Boolean(b)) => Ok((!b).to_expr()),
        _ => Ok(false.to_expr()),
    }
}

pub fn equal(args: Args) -> Result<Expr, EvalErr> {
    let first_two = args
        .into_iter()
        .own_two_or_else(|| EvalErr::InvalidArgs("'equal?'. expected two arguments."))?;
    match first_two {
        (
            Expr::Atom(Token::Str(x) | Token::Symbol(x)),
            Expr::Atom(Token::Str(y) | Token::Symbol(y)),
        ) => Ok((x == y).to_expr()),
        (Expr::Atom(Token::Number(x)), Expr::Atom(Token::Number(y))) => Ok((x == y).to_expr()),
        _ => Ok(false.to_expr()),
    }
}
