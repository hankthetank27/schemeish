use std::env;
use std::error::Error;
use std::fs;
use std::process;

use schemeish::enviroment::{Env, EnvRef};
use schemeish::evaluator::eval;
use schemeish::lexer::tokenize;
use schemeish::parser::parse;
use schemeish::parser::Expr;

fn main() {
    let mut args = env::args();

    let file = read(&mut args).unwrap_or_else(|err| {
        eprint!("{err}");
        process::exit(1);
    });

    let tokens = tokenize(&file);
    let exprs = parse(tokens);
    let global_env = Env::new(EnvRef::new_empty());
    let global_ref = EnvRef::new(global_env);
    for exp in exprs.iter() {
        let evalulated = eval(exp, global_ref.clone_rc());
        if let Expr::Proc(p) = evalulated {
            println!("{:?}", p.printable())
        } else {
            println!("{:?}", evalulated)
        }
    }
}

fn read<T>(args: &mut T) -> Result<String, Box<dyn Error>>
where
    T: Iterator<Item = String>,
{
    args.next();

    let path = match args.next() {
        Some(path) => path,
        None => return Err("Usage: rsscheme file_path.scm".into()),
    };

    Ok(fs::read_to_string(path)?)
}

#[cfg(test)]
mod test {
    use schemeish::{lexer::Token, parser::Expr};

    use super::*;

    #[test]
    fn can_do_arithemtic() {
        let scm = "(+ 1 (+ (+ 1 2)(- 2 1) 6 7 8 (- 3 2)))";
        let tokens = tokenize(scm);
        let exprs = parse(tokens);
        let global_env = Env::new(EnvRef::new_empty());
        let global_ref = EnvRef::new(global_env);
        let evalulated = eval(exprs.get(0).unwrap(), global_ref);
        assert_eq!(evalulated, Expr::Atom(Token::Number(27.0)));
    }
}
