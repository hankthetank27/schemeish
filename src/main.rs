use std::env;
use std::error::Error;
use std::fs;
use std::process;

mod tokenize;
use tokenize::tokenize;

fn main() {
    let args = env::args();

    let file = read(args).unwrap_or_else(|err| {
        eprint!("{err}");
        process::exit(1);
    });

    let tokens = tokenize(file.chars().peekable(), vec![]);
    dbg!(tokens);
}

fn read(mut args: impl Iterator<Item = String>) -> Result<String, Box<dyn Error>> {
    args.next();

    let path = match args.next() {
        Some(path) => path,
        None => return Err("Useage: rsscheme file_path.scm".into()),
    };

    Ok(fs::read_to_string(path)?)
}
