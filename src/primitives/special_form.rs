use crate::{
    error::EvalErr,
    evaluator::{eval, Args},
    lexer::Token,
    parser::Expr,
    primitives::utils::{GetVals, IterInnerVal, ToExpr},
    procedure::Compound,
};

pub fn define(args: Args) -> Result<Expr, EvalErr> {
    let env = args.env();
    let (identifier, mut args) = args.into_iter().get_one_and_rest()?;
    match identifier {
        //bind var
        Expr::Atom(Token::Symbol(identifier)) => {
            let value = args.next().expect("Expected value for variable");
            let val_expr = eval(value, &env)?;
            env.insert_val(identifier.to_string(), val_expr)
        }

        //bind proc
        Expr::List(first_expr) => {
            let mut first_expr = first_expr.into_strings()?.into_iter();
            let proc_name = first_expr.next().expect("Expected identifier for proc");
            let proc_args = first_expr.collect::<Vec<String>>();
            let proc_body = args.collect();

            let proc = Compound::new(proc_body, proc_args, env.clone_rc()).to_expr();
            env.insert_val(proc_name.to_string(), proc)
        }
        identifier => Err(EvalErr::TypeError(("symbol or list", identifier))), //panic!("Failed to define {:?}", identifier),
    }
}

pub fn lambda(args: Args) -> Result<Expr, EvalErr> {
    let env = args.env();
    let (first_expr, args) = args.into_iter().get_one_and_rest()?;
    match first_expr {
        Expr::List(first_expr) => {
            let proc_args = first_expr.into_strings()?;
            let proc_body = args.collect();

            Ok(Compound::new(proc_body, proc_args, env.clone_rc()).to_expr())
        }
        first_expr => Err(EvalErr::TypeError(("list", first_expr))), //panic!("Failed to define {:?}", identifier),
    }
}

pub fn if_statement(args: Args) -> Result<Expr, EvalErr> {
    let env = args.env();
    let mut args = args.into_iter();
    let (predicate, consequence, alternative) = args.get_three()?;
    match eval(predicate, &env)? {
        Expr::Atom(Token::Boolean(true)) => eval(consequence, &env),
        Expr::Atom(Token::Boolean(false)) => eval(alternative, &env),
        pred => Err(EvalErr::TypeError(("bool", pred))), //panic!("Failed to define {:?}", identifier),
    }
}
