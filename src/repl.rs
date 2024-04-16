use std::io::{self, Write};

use crate::enviroment::EnvRef;
use crate::evaluator;
use crate::lexer::tokenize;
use crate::parser::parse;
use crate::parser::Expr;

pub fn run() {
    println!("Schemeish v0.0.1");
    println!("Welcome :)");

    let global = EnvRef::global();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut exprs = String::new();

        if io::stdin().read_line(&mut exprs).is_ok() {
            let tokens = match tokenize(&exprs) {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            let exprs = match parse(tokens) {
                Ok(x) => x,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            for exp in exprs.into_iter() {
                match evaluator::eval(exp, &global) {
                    Ok(evalulated) => {
                        if let Expr::Proc(p) = evalulated {
                            println!("{:?}", p.printable())
                        } else {
                            println!("{:?}", evalulated)
                        }
                    }
                    Err(err) => eprintln!("{err}"),
                }
            }
        } else {
            eprint!("Error reading line");
            continue;
        }
    }
}
