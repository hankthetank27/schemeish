use std::io::{self, Write};
use std::process;

use crate::enviroment::EnvRef;
use crate::evaluator;
use crate::lexer::TokenStream;
use crate::parser::Parser;
use crate::print::Print;

pub struct Repl {
    global_env: EnvRef,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            global_env: EnvRef::global(),
        }
    }

    pub fn run(&self) {
        println!("Schemeish v0.1.0");
        println!("Welcome :)");

        self.global_env.import_prelude().unwrap_or_else(|err| {
            eprintln!("{err}");
            process::exit(1);
        });

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut exprs = String::new();

            if io::stdin().read_line(&mut exprs).is_ok() {
                let tokens = match TokenStream::new(&exprs).collect_tokens() {
                    Ok(x) => x,
                    Err(err) => {
                        eprintln!("{err}");
                        continue;
                    }
                };

                let exprs = match Parser::new(tokens).parse() {
                    Ok(x) => x,
                    Err(err) => {
                        eprintln!("{err}");
                        continue;
                    }
                };

                for exp in exprs.into_iter() {
                    match evaluator::eval(exp, &self.global_env) {
                        Ok(evalulated) => evalulated.print(),
                        Err(err) => eprintln!("{err}"),
                    }
                }
            } else {
                eprint!("Error reading line");
                continue;
            }
        }
    }
}
