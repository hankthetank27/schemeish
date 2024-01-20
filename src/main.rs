use std::env;
use std::error::Error;
use std::fs;
use std::process;

use rsscheme::lexer::tokenize;
use rsscheme::parser::parse;

fn main() {
    let mut args = env::args();

    let file = read(&mut args).unwrap_or_else(|err| {
        eprint!("{err}");
        process::exit(1);
    });

    let tokens = tokenize(&file);
    let parsed = parse(tokens);
    dbg!(parsed);
}

fn read<T>(args: &mut T) -> Result<String, Box<dyn Error>>
where
    T: Iterator<Item = String>,
{
    args.next();

    let path = match args.next() {
        Some(path) => path,
        None => return Err("Useage: rsscheme file_path.scm".into()),
    };

    Ok(fs::read_to_string(path)?)
}
