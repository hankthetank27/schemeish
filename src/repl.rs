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
        let mut exprs = String::new();

        print!("> ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut exprs)
            .expect("Failed to read line");

        let tokens = tokenize(&exprs);
        let exprs = parse(tokens);
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
    }
}
